use std::process::Command;

use anyhow::{Error, Result, anyhow};

pub trait WhichInterpreter {
    /// Get the intepreter of the corresponding data structure
    fn get_intepreter(&self) -> String;
}

/// Represent various kind of shells
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[allow(clippy::doc_markdown)]
pub enum ShellType {
    /// Bourne Again SHell (bash)
    Bash,
    /// Cmd (Command Prompt)
    Cmd,
}

impl WhichInterpreter for ShellType {
    fn get_intepreter(&self) -> String {
        match self {
            ShellType::Bash => String::from("sh"),
            ShellType::Cmd => String::from("cmd"),
        }
    }
}

pub fn execute_shell_script(shell_script: &str) -> Result<(), Error> {
    if cfg!(target_os = "windows") {
        match Command::new("cmd").args(["/C", shell_script]).status() {
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

    match Command::new("sh").arg(shell_script).status() {
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
