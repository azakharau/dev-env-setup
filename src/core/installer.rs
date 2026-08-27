use std::fmt;
use std::process::Command;

use super::error::InstallError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsKind {
    MacOS,
    Linux,
}

impl OsKind {
    pub fn detect() -> Option<Self> {
        match std::env::consts::OS {
            "macos" => Some(Self::MacOS),
            "linux" => Some(Self::Linux),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MacOS => "macos",
            Self::Linux => "linux",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::MacOS => "macOS",
            Self::Linux => "Linux",
        }
    }

    pub fn installer(self) -> Option<InstallerKind> {
        match self {
            Self::MacOS => Some(InstallerKind::Homebrew),
            Self::Linux => detect_linux_installer(),
        }
    }
}

impl fmt::Display for OsKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerKind {
    Homebrew,
    Pacman,
    Apt,
}

impl InstallerKind {
    pub fn command(self) -> &'static str {
        match self {
            Self::Homebrew => "brew",
            Self::Pacman => "pacman",
            Self::Apt => "apt",
        }
    }

    pub fn as_config_str(self) -> &'static str {
        match self {
            Self::Homebrew => "homebrew",
            Self::Pacman => "pacman",
            Self::Apt => "apt",
        }
    }

    pub fn needs_sudo(self) -> bool {
        matches!(self, Self::Pacman | Self::Apt)
    }

    /// Build install command. Returns (program, static_flags) — caller appends package name.
    fn install_cmd(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Homebrew => ("brew", &["install"]),
            Self::Pacman => ("sudo", &["pacman", "-S", "--noconfirm"]),
            Self::Apt => ("sudo", &["apt", "install", "-y"]),
        }
    }
}

impl fmt::Display for InstallerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.command())
    }
}

pub fn install_package(installer: InstallerKind, package: &str) -> Result<(), InstallError> {
    let (prog, flags) = installer.install_cmd();

    let output = Command::new(prog)
        .args(flags)
        .arg(package)
        .output()
        .map_err(InstallError::Io)?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(InstallError::CommandFailed(format!(
            "{installer} install {package} failed: {stderr}"
        )))
    }
}

pub fn run_script(package: &str, script: &[String]) -> Result<(), InstallError> {
    let (first, rest) = script
        .split_first()
        .ok_or_else(|| InstallError::ScriptFailed {
            package: package.into(),
            reason: "empty script".into(),
        })?;

    let output = Command::new(first)
        .args(rest)
        .output()
        .map_err(InstallError::Io)?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(InstallError::ScriptFailed {
            package: package.into(),
            reason: stderr.into_owned(),
        })
    }
}

fn detect_linux_installer() -> Option<InstallerKind> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;

    let mut is_arch = false;

    for line in content.lines() {
        if let Some(val) = line.strip_prefix("ID=") {
            let val = val.trim_matches('"');
            is_arch |= val.eq_ignore_ascii_case("arch");
        } else if let Some(val) = line.strip_prefix("ID_LIKE=") {
            let val = val.trim_matches('"');
            for part in val.split_ascii_whitespace() {
                is_arch |= part.eq_ignore_ascii_case("arch");
            }
        }
    }

    if is_arch {
        Some(InstallerKind::Pacman)
    } else {
        Some(InstallerKind::Apt)
    }
}
