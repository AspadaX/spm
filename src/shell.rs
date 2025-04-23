use std::{fmt::Display, process::Command};

use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};

/// Represent various kind of shells
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[allow(clippy::doc_markdown)]
pub enum ShellType {
    /// Sh
    Sh,
    /// Bourne Again SHell (bash)
    Bash,
    /// Zsh
    Zsh,
    /// Cmd (Command Prompt)
    Cmd,
}

impl ShellType {
    /// Returns the shebang line for the corresponding shell interpreter
    pub fn get_shebang(&self) -> &'static str {
        match self {
            ShellType::Bash => "#!/usr/bin/env bash",
            ShellType::Cmd => "#!/usr/bin/env cmd",
            ShellType::Sh => "#!/usr/bin/env sh",
            ShellType::Zsh => "#!/usr/bin/env zsh",
        }
    }
}

impl From<String> for ShellType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "sh" => ShellType::Sh,
            "bash" => ShellType::Bash,
            "zsh" => ShellType::Zsh,
            "cmd" => ShellType::Cmd,
            _ => panic!(
                "Unsupported shell type: {}. Please submit an issue in the repository.",
                s
            ),
        }
    }
}

impl std::str::FromStr for ShellType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sh" => Ok(ShellType::Sh),
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "cmd" => Ok(ShellType::Cmd),
            _ => Err(anyhow!(
                "Unsupported shell type: {}. Please submit an issue in the repository.",
                s
            )),
        }
    }
}

impl Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shell_name = match self {
            ShellType::Bash => "bash",
            ShellType::Cmd => "cmd",
            ShellType::Sh => "sh",
            ShellType::Zsh => "zsh",
        };
        write!(f, "{}", shell_name)
    }
}

pub fn execute_shell_script(shell_script: &str) -> Result<(), Error> {
    let script_path: &std::path::Path = std::path::Path::new(shell_script);
    let script_dir: &std::path::Path = script_path.parent().unwrap_or_else(|| std::path::Path::new("."));

    if cfg!(target_os = "windows") {
        match Command::new("cmd").args(["/C", shell_script]).current_dir(script_dir).status() {
            Ok(status) if !status.success() => {
                return Err(anyhow!(
                    "Windows CMD interpreter exited with a non-zero status"
                ));
            }
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow!("Failed to start Windows CMD interpreter: {}", e));
            }
        }
    }

    match Command::new("sh").arg(shell_script).current_dir(script_dir).status() {
        Ok(status) if !status.success() => {
            return Err(anyhow!("Shell interpreter exited with a non-zero status"));
        }
        Ok(_) => {}
        Err(e) => {
            return Err(anyhow!("Failed to start shell interpreter: {}", e));
        }
    }

    Ok(())
}
