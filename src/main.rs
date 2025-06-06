mod arguments;
mod package;
mod shell;
mod utilities;
mod display_control;
mod properties;

use std::path::{Path, PathBuf};

use arguments::{Arguments, Commands};
use clap::{Parser, crate_version};

use display_control::display_message;
use package::{Package, PackageManager};
use utilities::{cleanup_temp_repository, execute_run_command, fetch_remote_git_repository, is_git_repository_link, show_packages};

fn main() {
    // Parse command line arguments
    let arguments: Arguments = Arguments::parse();
    // Initialize a package manager
    let package_manager: PackageManager = match PackageManager::new() {
        Ok(result) => result,
        Err(error) => {
            display_message(display_control::Level::Error, &format!("{}", error.to_string()));
            return;
        }
    };

    // Check if the binary directory is in the user's PATH
    utilities::check_bin_directory_in_path(&package_manager);
    
    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => match execute_run_command(&package_manager, subcommand.expression, &subcommand.args) {
            Ok(_) => {}
            Err(error) => display_message(display_control::Level::Error, &format!("{}", error.to_string())),
        },
        Commands::Install(subcommand) => {
            let package_path: PathBuf;
            let mut is_move: bool = false;
            let mut temp_path_opt: Option<PathBuf> = None;
            
            // Determine whether this is a remote installation, or local
            if is_git_repository_link(&subcommand.path) {
                // Create a subcommand for handling git repository installations
                let cmd_parts: Vec<&str> = subcommand.path.split("/").collect();
                if cmd_parts.len() < 2 {
                    display_message(display_control::Level::Error, "Invalid Git repository format. Expected: username/repo");
                    return;
                }
                
                // Fetch the repository to a temporary directory
                package_path = match fetch_remote_git_repository(&subcommand.base_url, &subcommand.path) {
                    Ok(result) => {
                        // Store the temp path for later cleanup
                        temp_path_opt = Some(result.clone());
                        result
                    },
                    Err(error) => {
                        display_message(display_control::Level::Error, &format!("{}", error.to_string()));
                        return;
                    },
                };
                
                // Move the local git repository for installations
                is_move = true;
            } else {
                package_path = Path::new(&subcommand.path).to_path_buf();
            }
            
            // Install the package
            let install_result = package_manager.install_package(&package_path, is_move, subcommand.force);
            
            // Clean up the temporary directory if used
            if let Some(temp_path) = temp_path_opt {
                if let Err(cleanup_err) = cleanup_temp_repository(&temp_path) {
                    display_message(display_control::Level::Warn, &format!("Failed to clean up temporary directory: {}", cleanup_err));
                }
            }
            
            // Handle installation result
            match install_result {
                Ok(_) => display_message(display_control::Level::Logging, "Package installation succeeded."),
                Err(error) => display_message(display_control::Level::Error, &format!("{}", error.to_string())),
            }
        },
        Commands::List(_) => {
            match package_manager.get_installed_packages() {
                Ok(packages_metadata) => {
                    show_packages(&packages_metadata);
                },
                Err(error) => {
                    display_message(display_control::Level::Error, &format!("Error retrieving installed packages: {}", error.to_string()));
                }
            };
        }
        Commands::Uninstall(subcommand) => {
            match package_manager.uninstall_package_by_name(subcommand.expression) {
                Ok(_) => display_message(display_control::Level::Logging, "Package uninstalled successfully."),
                Err(error) => display_message(display_control::Level::Error, &format!("Error uninstalling package: {}", error.to_string())),
            }
        }
        Commands::Check(_) => {
            display_message(display_control::Level::Logging, "The 'Check' feature is still under development.");
        }
        Commands::New(subcommand) => {
            let working_directory: PathBuf = Path::new("./").join(&subcommand.name);
            match std::fs::create_dir(&working_directory) {
                Ok(_) => {}
                Err(error) => display_message(display_control::Level::Error, &format!("{}", error.to_string())),
            };

            let package = match subcommand.namespace {
                Some(namespace) => Package::new_with_namespace(subcommand.name, namespace, subcommand.lib, subcommand.interpreter.into()),
                None => Package::new(subcommand.name, subcommand.lib, subcommand.interpreter.into()),
            };
            match package_manager.create_package(working_directory.as_path(), &package) {
                Ok(_) => display_message(display_control::Level::Logging, "Package created successfully."),
                Err(error) => display_message(display_control::Level::Error, &format!("{}", error.to_string())),
            };
        }
        Commands::Init(subcommand) => {
            let working_directory: &Path = Path::new("./");
            let folder_name = working_directory
                .canonicalize()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
                
            let package = match subcommand.namespace {
                Some(namespace) => Package::new_with_namespace(folder_name, namespace, subcommand.lib, subcommand.interpreter.into()),
                None => Package::new(folder_name, subcommand.lib, subcommand.interpreter.into()),
            };

            match package_manager.create_package(working_directory, &package) {
                Ok(_) => display_message(display_control::Level::Logging, "Package created successfully."),
                Err(error) => display_message(display_control::Level::Error, &format!("{}", error.to_string())),
            };
        }
        Commands::Version(_) => {
            display_message(display_control::Level::Logging, &format!("Shell Package Manager (spm) version: {}", crate_version!()));
        }
    }

    return;
}
