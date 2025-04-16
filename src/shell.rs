use std::process::Command;

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

pub fn execute_shell_script(shell_script: &str) {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", shell_script])
            .status()
            .expect("Windows Bash Intepreter failed to start");
        return;
    }

    Command::new("sh")
        .arg(shell_script)
        .status()
        .expect("Shell intepreter failed to start");
}
