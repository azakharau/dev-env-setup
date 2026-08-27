use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Install error: {0}")]
    Install(#[from] InstallError),

    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Unsupported OS: {0}")]
    UnsupportedOs(String),
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Package manager command failed: {0}")]
    CommandFailed(String),

    #[error("Script execution failed for '{package}': {reason}")]
    ScriptFailed { package: String, reason: String },

    #[error("IO error during install: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git command failed: {0}")]
    CommandFailed(String),

    #[error("Invalid repository URL: {0}")]
    InvalidUrl(String),

    #[error("SSH key setup failed: {0}")]
    SshSetupFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
