mod arguments;
mod package;
mod shell;

use std::path::{Path, PathBuf};

use arguments::{Arguments, Commands};
use clap::Parser;
use console::Term;
use package::{Package, PackageManager};
use shell::execute_shell_script;

fn main() {
    // Create a terminal display
    let terminal: Term = Term::stdout();
    // Parse command line arguments
    let arguments = Arguments::parse();
    // Initialize a package manager
    let package_manager = match PackageManager::new() {
        Ok(result) => result,
        Err(error) => {
            terminal
                .write_line(&format!("{}", error.to_string()))
                .unwrap();
            return;
        }
    };

    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => {
            execute_shell_script(&subcommand.expression);
        }
        Commands::Install(subcommand) => {
            match package_manager.install_package(Path::new(&subcommand.path), false) {
                Ok(_) => terminal
                    .write_line("Package installation succeeded.")
                    .unwrap(),
                Err(error) => terminal
                    .write_line(&format!("{}", error.to_string()))
                    .unwrap(),
            }
        }
        Commands::List(_) => {}
        Commands::Uninstall(subcommand) => {}
        Commands::Check(subcommand) => {}
        Commands::New(subcommand) => {
            let working_directory: PathBuf = Path::new("./").join(&subcommand.name);
            match std::fs::create_dir(&working_directory) {
                Ok(_) => {}
                Err(error) => terminal
                    .write_line(&format!("{}", error.to_string()))
                    .unwrap(),
            };

            match package_manager.create_package(
                working_directory.as_path(),
                &Package::new(subcommand.name, subcommand.lib),
            ) {
                Ok(_) => terminal
                    .write_line("Package created successfully.")
                    .unwrap(),
                Err(error) => terminal
                    .write_line(&format!("{}", error.to_string()))
                    .unwrap(),
            };

            return;
        }
        Commands::Init(subcommand) => {
            let working_directory: &Path = Path::new("./");
            match package_manager.create_package(
                working_directory,
                &Package::new(
                    working_directory
                        .canonicalize()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    subcommand.lib,
                ),
            ) {
                Ok(_) => terminal
                    .write_line("Package created successfully.")
                    .unwrap(),
                Err(error) => terminal
                    .write_line(&format!("{}", error.to_string()))
                    .unwrap(),
            };

            return;
        }
        Commands::Version(_) => {}
    }
}
