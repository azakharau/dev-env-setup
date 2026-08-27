use std::path::PathBuf;

use anyhow::{Context, Result, bail};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let config_arg = parse_config_arg()?;
    let config_path = resolve_config_path(config_arg)?;

    let config = dev_env_setup::core::config::AppConfig::load(&config_path)
        .with_context(|| format!("Failed to load config from {}", config_path.display()))?;

    dev_env_setup::tui::run(config, config_path)
}

fn parse_config_arg() -> Result<Option<PathBuf>> {
    let mut args = std::env::args().skip(1);
    let mut config_path = None;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }

        if arg == "--config" || arg == "-c" {
            let Some(path) = args.next() else {
                bail!("Missing path after {arg}");
            };
            if config_path.replace(PathBuf::from(path)).is_some() {
                bail!("Config path was provided more than once");
            }
            continue;
        }

        if let Some(rest) = arg.strip_prefix("--config=") {
            if config_path.replace(PathBuf::from(rest)).is_some() {
                bail!("Config path was provided more than once");
            }
            continue;
        }

        if let Some(rest) = arg.strip_prefix("-c=") {
            if config_path.replace(PathBuf::from(rest)).is_some() {
                bail!("Config path was provided more than once");
            }
            continue;
        }

        bail!("Unknown argument: {arg}. Supported: --config <path> or -c <path>");
    }

    Ok(config_path)
}

fn resolve_config_path(config_arg: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = config_arg {
        let path = absolutize_path(dev_env_setup::core::config::expand_tilde(
            path.to_string_lossy().as_ref(),
        ))?;

        if !path.is_file() {
            bail!(
                "Config file not found at {}\nUse --config /path/to/config.toml or create the file.",
                path.display()
            );
        }

        return Ok(path);
    }

    let default = default_config_path();
    if default.is_file() {
        return Ok(default);
    }

    #[cfg(debug_assertions)]
    {
        let cwd = std::env::current_dir().context("Failed to determine current directory")?;
        let dev_config = cwd.join("config.toml");
        let cargo_toml = cwd.join("Cargo.toml");

        if cargo_toml.is_file() && dev_config.is_file() {
            return Ok(dev_config);
        }

        bail!(
            "No config file found.\nTried default path: {}\nAlso tried dev fallback: {}\nCreate the file or run dev-forge --config /path/to/config.toml",
            default.display(),
            dev_config.display()
        );
    }

    #[cfg(not(debug_assertions))]
    {
        bail!(
            "No config file found.\nTried default path: {}\nCreate the file or run dev-forge --config /path/to/config.toml",
            default.display()
        );
    }
}

fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .map(|home| home.join(".config").join("dev-forge").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("~/.config/dev-forge/config.toml"))
}

fn absolutize_path(path: PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path);
    }

    Ok(std::env::current_dir()
        .context("Failed to determine current directory")?
        .join(path))
}

fn print_usage() {
    eprintln!("dev-forge");
    eprintln!("Usage: dev-forge [--config <path> | -c <path>]");
    eprintln!("Default config: ~/.config/dev-forge/config.toml");
}
