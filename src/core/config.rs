use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::error::ConfigError;
use super::installer::OsKind;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub configs_repo: String,
    #[serde(default = "default_configs_path")]
    pub configs_path: String,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub configs: Vec<DotfileConfig>,
}

fn default_configs_path() -> String {
    "~/cfg".into()
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn required_deps(&self) -> impl Iterator<Item = &Dependency> {
        self.dependencies.iter().filter(|d| d.required)
    }

    pub fn optional_deps(&self) -> impl Iterator<Item = &Dependency> {
        self.dependencies.iter().filter(|d| !d.required)
    }

    pub fn configs_for_os(&self, os: OsKind) -> impl Iterator<Item = &DotfileConfig> {
        let os_str = os.as_str();
        self.configs
            .iter()
            .filter(move |c| c.os.iter().any(|o| o == os_str))
    }

    pub fn resolved_configs_path(&self) -> PathBuf {
        expand_tilde(&self.configs_path)
    }
}

#[derive(Debug, Deserialize)]
pub struct Dependency {
    pub name: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub installers: Vec<String>,
    pub check: Option<String>,
    pub script: Option<Vec<String>>,
}

fn default_category() -> String {
    "other".into()
}

impl Dependency {
    pub fn is_installed(&self) -> bool {
        let check_cmd = self
            .check
            .as_deref()
            .map(std::borrow::Cow::Borrowed)
            .unwrap_or_else(|| std::borrow::Cow::Owned(format!("which {}", self.name)));

        Command::new("sh")
            .args(["-c", &check_cmd])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    pub fn supports_installer(&self, installer: &str) -> bool {
        self.installers.iter().any(|i| i == installer)
    }
}

#[derive(Debug, Deserialize)]
pub struct DotfileConfig {
    pub name: String,
    pub source: String,
    pub target: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub os: Vec<String>,
}

impl DotfileConfig {
    pub fn resolved_target(&self) -> PathBuf {
        match &self.target {
            Some(t) => expand_tilde(t),
            None => {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                home.join(".config").join(&self.name)
            }
        }
    }

    pub fn resolved_source(&self, configs_path: &Path) -> PathBuf {
        configs_path.join(&self.source)
    }
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        home.join(rest)
    } else if path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
    } else {
        PathBuf::from(path)
    }
}
