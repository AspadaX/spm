use std::process::Command;

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