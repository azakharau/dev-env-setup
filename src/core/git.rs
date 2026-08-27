use std::path::Path;
use std::process::Command;

use super::error::GitError;

pub fn clone_or_pull(repo_url: &str, target: &Path) -> Result<(), GitError> {
    if target.join(".git").exists() {
        let output = Command::new("git")
            .args(["pull"])
            .current_dir(target)
            .output()
            .map_err(GitError::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(GitError::CommandFailed(format!(
                "git pull failed: {stderr}"
            )))
        }
    } else {
        let target_str = target.to_string_lossy();
        let output = Command::new("git")
            .args(["clone", repo_url, &target_str])
            .output()
            .map_err(GitError::Io)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(GitError::CommandFailed(format!(
                "git clone failed: {stderr}"
            )))
        }
    }
}

pub fn has_ssh_key() -> bool {
    let home = dirs::home_dir().unwrap_or_default();
    let ssh_dir = home.join(".ssh");
    ssh_dir.join("id_ed25519").exists() || ssh_dir.join("id_rsa").exists()
}

pub fn deploy_config(source: &Path, target: &Path) -> Result<(), GitError> {
    if !source.exists() {
        return Err(GitError::CommandFailed(format!(
            "Config source does not exist: {}",
            source.display()
        )));
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(GitError::Io)?;
    }

    if target.exists() || target.symlink_metadata().is_ok() {
        if target.is_dir() && !target.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
            std::fs::remove_dir_all(target).map_err(GitError::Io)?;
        } else {
            std::fs::remove_file(target).map_err(GitError::Io)?;
        }
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).map_err(GitError::Io)?;

    Ok(())
}
