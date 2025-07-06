mod arguments;
mod display_control;
mod program;
mod properties;
mod shell;
mod utilities;

use std::path::{Path, PathBuf};

use arguments::{Arguments, Commands};
use clap::{Parser, crate_version};

use display_control::display_message;
use program::{Program, ProgramManager};
use utilities::{
    execute_run_command, show_programs,
};

fn main() {
    // Parse command line arguments
    let arguments: Arguments = Arguments::parse();
    // Initialize a program manager
    let program_manager: ProgramManager = match ProgramManager::new() {
        Ok(result) => result,
        Err(error) => {
            display_message(
                display_control::Level::Error,
                &format!("{}", error.to_string()),
            );
            return;
        }
    };

    // Check if the binary directory is in the user's PATH
    let _ = utilities::check_bin_directory_in_path();

    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => {
            match execute_run_command(&program_manager, subcommand.expression, &subcommand.args) {
                Ok(_) => {}
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("{}", error.to_string()),
                ),
            }
        }
        Commands::Install(subcommand) => {
            // Check if the path is a Git URL
            if subcommand.path.starts_with("http://") || subcommand.path.starts_with("https://") || subcommand.path.starts_with("git@") {
                match program_manager.install_from_git(&subcommand.path, subcommand.force) {
                    Ok(_) => display_message(
                        display_control::Level::Logging,
                        "Programs from Git repository installed successfully!",
                    ),
                    Err(error) => display_message(
                        display_control::Level::Error,
                        &format!("Error installing programs from Git repository: {}", error.to_string()),
                    ),
                }
            } else {
                let program_path = Path::new(&subcommand.path).to_path_buf();

                // Install the program
                match program_manager.install_program(&program_path, subcommand.force) {
                    Ok(_) => display_message(
                        display_control::Level::Logging,
                        "Program installation succeeded.",
                    ),
                    Err(error) => display_message(
                        display_control::Level::Error,
                        &format!("{}", error.to_string()),
                    ),
                }
            }
        }
        Commands::List(_) => {
            match program_manager.get_installed_programs() {
                Ok(programs) => {
                    show_programs(&programs);
                }
                Err(error) => {
                    display_message(
                        display_control::Level::Error,
                        &format!("Error retrieving installed programs: {}", error.to_string()),
                    );
                }
            };
        }
        Commands::Uninstall(subcommand) => {
            match program_manager.uninstall_program_by_name(subcommand.expression) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    "Program uninstalled successfully.",
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("Error uninstalling program: {}", error.to_string()),
                ),
            }
        }
        Commands::Check(_) => {
            display_message(
                display_control::Level::Logging,
                "The 'Check' feature is still under development.",
            );
        }
        Commands::New(subcommand) => {
            let program_file_path: PathBuf =
                Path::new("./").join(format!("{}.sh", &subcommand.name));
            let program = Program::new(subcommand.name, crate::shell::ShellType::Sh);

            match program_manager.create_program(&program_file_path, &program) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    "Program created successfully.",
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("{}", error.to_string()),
                ),
            };
        }
        Commands::Version(_) => {
            display_message(
                display_control::Level::Logging,
                &format!("Shell Program Manager (spm) version: {}", crate_version!()),
            );
        }
    }

    return;
}
