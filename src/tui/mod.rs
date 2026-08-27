mod layout;
mod screens;

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::prelude::*;

use crate::core::config::AppConfig;
use crate::core::git;
use crate::core::installer::{self, InstallerKind, OsKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Welcome,
    InstallingRequired,
    SelectDeps,
    InstallingSelected,
    SelectConfigs,
    DeployingConfigs,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Pending,
    InProgress,
    Done,
    AlreadyDone,
    Failed,
}

#[derive(Debug)]
struct DepItem {
    name: String,
    category: String,
    description: String,
    selected: bool,
    status: Status,
    installers: Vec<String>,
    script: Option<Vec<String>>,
    required: bool,
}

impl DepItem {
    fn from_dep(dep: &crate::core::config::Dependency) -> Self {
        let already = dep.is_installed();
        Self {
            name: dep.name.clone(),
            category: dep.category.clone(),
            description: dep.description.clone(),
            selected: dep.required,
            status: if already {
                Status::AlreadyDone
            } else {
                Status::Pending
            },
            installers: dep.installers.clone(),
            script: dep.script.clone(),
            required: dep.required,
        }
    }
}

#[derive(Debug)]
struct ConfigItem {
    name: String,
    description: String,
    source: String,
    target_display: String,
    selected: bool,
    status: Status,
}

impl ConfigItem {
    fn from_config(cfg: &crate::core::config::DotfileConfig) -> Self {
        let target = cfg.resolved_target();
        Self {
            name: cfg.name.clone(),
            description: cfg.description.clone(),
            source: cfg.source.clone(),
            target_display: target.to_string_lossy().into_owned(),
            selected: true,
            status: Status::Pending,
        }
    }
}

#[derive(Debug, Clone)]
enum LogLevel {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone)]
struct LogEntry {
    level: LogLevel,
    message: String,
}

enum WorkerMsg {
    DepStarted(usize),
    DepFinished(usize, bool),
    ConfigStarted(usize),
    ConfigFinished(usize, bool),
    Log(LogLevel, String),
    Done,
}

struct App {
    screen: Screen,
    os: OsKind,
    installer: Option<InstallerKind>,
    config: AppConfig,
    config_path: String,
    git_available: bool,
    package_manager_available: bool,
    ssh_key_available: bool,
    local_configs_repo_ready: bool,
    deps: Vec<DepItem>,
    configs: Vec<ConfigItem>,
    cursor: usize,
    log: Vec<LogEntry>,
    should_quit: bool,
    rx: Option<mpsc::Receiver<WorkerMsg>>,
    progress_current: usize,
    progress_total: usize,
}

impl App {
    fn new(config: AppConfig, config_path: &Path, os: OsKind) -> Self {
        let installer = os.installer();
        let git_available = command_available("git");
        let package_manager_available =
            installer.is_some_and(|kind| command_available(kind.command()));
        let ssh_key_available = git::has_ssh_key();
        let local_configs_repo_ready = config.resolved_configs_path().join(".git").exists();
        let deps = config.dependencies.iter().map(DepItem::from_dep).collect();
        let configs = config
            .configs_for_os(os)
            .map(ConfigItem::from_config)
            .collect();

        Self {
            screen: Screen::Welcome,
            os,
            installer,
            config,
            config_path: config_path.display().to_string(),
            git_available,
            package_manager_available,
            ssh_key_available,
            local_configs_repo_ready,
            deps,
            configs,
            cursor: 0,
            log: Vec::new(),
            should_quit: false,
            rx: None,
            progress_current: 0,
            progress_total: 0,
        }
    }

    fn has_optional(&self) -> bool {
        self.deps.iter().any(|d| !d.required)
    }

    fn optional_indices(&self) -> Vec<usize> {
        self.deps
            .iter()
            .enumerate()
            .filter(|(_, d)| !d.required)
            .map(|(i, _)| i)
            .collect()
    }

    fn spawn_dep_install(&mut self, filter: impl Fn(&DepItem) -> bool, screen: Screen) {
        let need_install: Vec<usize> = self
            .deps
            .iter()
            .enumerate()
            .filter(|(_, d)| filter(d) && d.status == Status::Pending)
            .map(|(i, _)| i)
            .collect();

        if need_install.is_empty() {
            self.log.push(LogEntry {
                level: LogLevel::Info,
                message: if screen == Screen::InstallingRequired {
                    "All required dependencies already installed.".into()
                } else {
                    "No optional dependencies to install.".into()
                },
            });
            if screen == Screen::InstallingRequired {
                self.advance_from_required();
            } else {
                self.advance_from_optional();
            }
            return;
        }

        self.screen = screen;
        self.progress_current = 0;
        self.progress_total = need_install.len();
        if screen == Screen::InstallingSelected {
            self.log.clear();
        }

        let installer = self.installer;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let items: Vec<_> = need_install
            .into_iter()
            .map(|i| {
                let d = &self.deps[i];
                (i, d.name.clone(), d.installers.clone(), d.script.clone())
            })
            .collect();

        thread::spawn(move || {
            for (idx, name, installers, script) in items {
                let _ = tx.send(WorkerMsg::DepStarted(idx));
                let _ = tx.send(WorkerMsg::Log(
                    LogLevel::Info,
                    format!("Installing {name}..."),
                ));

                let result = if let Some(ref script) = script {
                    installer::run_script(&name, script)
                } else if let Some(inst) = installer {
                    if installers.iter().any(|c| c == inst.as_config_str()) {
                        installer::install_package(inst, &name)
                    } else {
                        Err(crate::core::error::InstallError::CommandFailed(format!(
                            "No compatible installer for {name} on this OS"
                        )))
                    }
                } else {
                    Err(crate::core::error::InstallError::CommandFailed(
                        "No package manager detected".into(),
                    ))
                };

                match &result {
                    Ok(()) => {
                        let _ = tx.send(WorkerMsg::Log(
                            LogLevel::Success,
                            format!("{name} installed successfully"),
                        ));
                    }
                    Err(e) => {
                        let _ = tx.send(WorkerMsg::Log(
                            LogLevel::Error,
                            format!("{name} failed: {e}"),
                        ));
                    }
                }

                let _ = tx.send(WorkerMsg::DepFinished(idx, result.is_ok()));
            }

            let _ = tx.send(WorkerMsg::Done);
        });
    }

    fn start_required_install(&mut self) {
        self.spawn_dep_install(|d| d.required, Screen::InstallingRequired);
    }

    fn start_optional_install(&mut self) {
        self.spawn_dep_install(|d| !d.required && d.selected, Screen::InstallingSelected);
    }

    fn start_config_deploy(&mut self) {
        let need_deploy: Vec<usize> = self
            .configs
            .iter()
            .enumerate()
            .filter(|(_, c)| c.selected)
            .map(|(i, _)| i)
            .collect();

        if need_deploy.is_empty() {
            self.log.push(LogEntry {
                level: LogLevel::Info,
                message: "No configs selected for deployment.".into(),
            });
            self.screen = Screen::Summary;
            return;
        }

        self.screen = Screen::DeployingConfigs;
        self.progress_current = 0;
        self.progress_total = need_deploy.len() + 1;
        self.log.clear();

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let configs_repo = self.config.configs_repo.clone();
        let configs_path = self.config.resolved_configs_path();
        let items: Vec<_> = need_deploy
            .into_iter()
            .map(|i| {
                let c = &self.configs[i];
                (i, c.source.clone(), c.target_display.clone())
            })
            .collect();

        thread::spawn(move || {
            let _ = tx.send(WorkerMsg::Log(
                LogLevel::Info,
                format!("Syncing configs repo to {}...", configs_path.display()),
            ));

            let git_ok = match git::clone_or_pull(&configs_repo, &configs_path) {
                Ok(()) => {
                    let _ = tx.send(WorkerMsg::Log(
                        LogLevel::Success,
                        "Configs repo synced.".into(),
                    ));
                    true
                }
                Err(e) => {
                    let _ = tx.send(WorkerMsg::Log(
                        LogLevel::Error,
                        format!("Git sync failed: {e}"),
                    ));
                    false
                }
            };

            if !git_ok {
                for &(idx, _, _) in &items {
                    let _ = tx.send(WorkerMsg::ConfigFinished(idx, false));
                }
                let _ = tx.send(WorkerMsg::Done);
                return;
            }

            for (idx, source, target) in &items {
                let _ = tx.send(WorkerMsg::ConfigStarted(*idx));
                let src = configs_path.join(source);
                let tgt = std::path::PathBuf::from(target);

                let _ = tx.send(WorkerMsg::Log(
                    LogLevel::Info,
                    format!("Deploying {source} -> {target}..."),
                ));

                match git::deploy_config(&src, &tgt) {
                    Ok(()) => {
                        let _ = tx.send(WorkerMsg::Log(
                            LogLevel::Success,
                            format!("{source} deployed"),
                        ));
                        let _ = tx.send(WorkerMsg::ConfigFinished(*idx, true));
                    }
                    Err(e) => {
                        let _ = tx.send(WorkerMsg::Log(
                            LogLevel::Error,
                            format!("{source} failed: {e}"),
                        ));
                        let _ = tx.send(WorkerMsg::ConfigFinished(*idx, false));
                    }
                }
            }

            let _ = tx.send(WorkerMsg::Done);
        });
    }

    fn advance_from_required(&mut self) {
        if self.has_optional() {
            self.screen = Screen::SelectDeps;
            self.cursor = 0;
        } else {
            self.advance_from_optional();
        }
    }

    fn advance_from_optional(&mut self) {
        if self.configs.is_empty() {
            self.screen = Screen::Summary;
        } else {
            self.screen = Screen::SelectConfigs;
            self.cursor = 0;
        }
    }

    fn poll_worker(&mut self) {
        let Some(rx) = self.rx.take() else {
            return;
        };

        let mut disconnected = false;
        loop {
            match rx.try_recv() {
                Ok(msg) => self.handle_worker_msg(msg),
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        if !disconnected {
            self.rx = Some(rx);
        }
    }

    fn handle_worker_msg(&mut self, msg: WorkerMsg) {
        match msg {
            WorkerMsg::DepStarted(i) => {
                if let Some(d) = self.deps.get_mut(i) {
                    d.status = Status::InProgress;
                }
            }
            WorkerMsg::DepFinished(i, ok) => {
                if let Some(d) = self.deps.get_mut(i) {
                    d.status = if ok { Status::Done } else { Status::Failed };
                }
                self.progress_current += 1;
            }
            WorkerMsg::ConfigStarted(i) => {
                if let Some(c) = self.configs.get_mut(i) {
                    c.status = Status::InProgress;
                }
            }
            WorkerMsg::ConfigFinished(i, ok) => {
                if let Some(c) = self.configs.get_mut(i) {
                    c.status = if ok { Status::Done } else { Status::Failed };
                }
                self.progress_current += 1;
            }
            WorkerMsg::Log(level, message) => self.log.push(LogEntry { level, message }),
            WorkerMsg::Done => {
                self.rx = None;
                match self.screen {
                    Screen::InstallingRequired => self.advance_from_required(),
                    Screen::InstallingSelected => self.advance_from_optional(),
                    Screen::DeployingConfigs => self.screen = Screen::Summary,
                    _ => {}
                }
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.screen {
            Screen::Welcome => match code {
                KeyCode::Enter => self.start_required_install(),
                KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            },
            Screen::InstallingRequired | Screen::InstallingSelected | Screen::DeployingConfigs => {
                if code == KeyCode::Char('q') {
                    self.should_quit = true;
                }
            }
            Screen::SelectDeps => {
                let opt = self.optional_indices();
                if opt.is_empty() {
                    return;
                }

                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.cursor = self.cursor.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.cursor < opt.len() - 1 {
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        let i = opt[self.cursor];
                        self.deps[i].selected = !self.deps[i].selected;
                    }
                    KeyCode::Char('a') => {
                        let all = opt.iter().all(|&i| self.deps[i].selected);
                        for i in opt {
                            self.deps[i].selected = !all;
                        }
                    }
                    KeyCode::Enter => self.start_optional_install(),
                    KeyCode::Esc => {
                        self.cursor = 0;
                        self.screen = Screen::Welcome;
                    }
                    KeyCode::Char('q') => self.should_quit = true,
                    _ => {}
                }
            }
            Screen::SelectConfigs => {
                let len = self.configs.len();
                if len == 0 {
                    return;
                }

                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.cursor = self.cursor.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if self.cursor < len - 1 {
                            self.cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        self.configs[self.cursor].selected = !self.configs[self.cursor].selected;
                    }
                    KeyCode::Char('a') => {
                        let all = self.configs.iter().all(|c| c.selected);
                        for c in &mut self.configs {
                            c.selected = !all;
                        }
                    }
                    KeyCode::Enter => self.start_config_deploy(),
                    KeyCode::Esc => {
                        self.cursor = 0;
                        self.screen = Screen::SelectDeps;
                    }
                    KeyCode::Char('q') => self.should_quit = true,
                    _ => {}
                }
            }
            Screen::Summary => match code {
                KeyCode::Char('q') | KeyCode::Enter => self.should_quit = true,
                _ => {}
            },
        }
    }
}

fn command_available(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub fn run(config: AppConfig, config_path: std::path::PathBuf) -> Result<()> {
    let os = OsKind::detect().context("Unsupported operating system")?;

    if os == OsKind::Linux
        && let Some(inst) = os.installer()
        && inst.needs_sudo()
    {
        eprintln!("Caching sudo credentials (may prompt for password)...");
        let _ = std::process::Command::new("sudo").args(["-v"]).status();
    }

    let mut app = App::new(config, &config_path, os);

    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let tick_rate = Duration::from_millis(50);

    loop {
        terminal.draw(|frame| layout::draw(frame, &app))?;

        if app.should_quit {
            break;
        }

        app.poll_worker();

        if event::poll(tick_rate)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code);
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
