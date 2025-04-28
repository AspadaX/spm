mod arguments;
mod commons;
mod display_control;
mod package;
mod properties;
mod shell;

use std::path::{Path, PathBuf};

use arguments::{Arguments, Commands};
use clap::{Parser, crate_version};

use commons::utilities::{
    check_bin_directory_in_path, cleanup_temporary_repository, execute_run_command,
    extract_name_and_namespace, handle_installation_path, show_packages,
};
use display_control::display_message;
use package::{Package, PackageManager};

fn main() {
    // Parse command line arguments
    let arguments: Arguments = Arguments::parse();
    // Initialize a package manager
    let mut package_manager: PackageManager = match PackageManager::new() {
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
    check_bin_directory_in_path(&package_manager);

    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => {
            match execute_run_command(&package_manager, subcommand.expression, &subcommand.args) {
                Ok(_) => {}
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("{}", error.to_string()),
                ),
            }
        }
        Commands::Install(subcommand) => {
            let package_path: PathBuf;
            let mut is_move: bool = false;
            let mut temporary_path_opt: Option<PathBuf> = None;

            // Determine whether this is a remote installation, or local
            package_path = handle_installation_path(
                &subcommand.path,
                &subcommand.base_url,
                &mut temporary_path_opt,
                &mut is_move,
            )
            .1;

            // Install the package
            let install_result =
                package_manager.install_package(&package_path, is_move, subcommand.force);

            // Clean up the temporary directory if used
            if let Some(temporary_path) = temporary_path_opt {
                if let Err(cleanup_err) = cleanup_temporary_repository(&temporary_path) {
                    display_message(
                        display_control::Level::Warn,
                        &format!("Failed to clean up temporary directory: {}", cleanup_err),
                    );
                }
            }

            // Handle installation result
            match install_result {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    "Package installation succeeded.",
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("{}", error.to_string()),
                ),
            }
        }
        Commands::List(_) => {
            match package_manager.get_installed_packages() {
                Ok(packages_metadata) => {
                    show_packages(&packages_metadata);
                }
                Err(error) => {
                    display_message(
                        display_control::Level::Error,
                        &format!("Error retrieving installed packages: {}", error.to_string()),
                    );
                }
            };
        }
        Commands::Uninstall(subcommand) => {
            match package_manager.uninstall_package_by_name(subcommand.expression) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    "Package uninstalled successfully.",
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("Error uninstalling package: {}", error.to_string()),
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
            let working_directory: PathBuf = Path::new("./").join(&subcommand.name);
            match std::fs::create_dir(&working_directory) {
                Ok(_) => {}
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("{}", error.to_string()),
                ),
            };

            let package = match subcommand.namespace {
                Some(namespace) => Package::new_with_namespace(
                    subcommand.name,
                    namespace,
                    subcommand.lib,
                    subcommand.interpreter.into(),
                ),
                None => Package::new(
                    subcommand.name,
                    subcommand.lib,
                    subcommand.interpreter.into(),
                ),
            };
            match package_manager.create_package(working_directory.as_path(), &package) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    "Package created successfully.",
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("{}", error.to_string()),
                ),
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
                Some(namespace) => Package::new_with_namespace(
                    folder_name,
                    namespace,
                    subcommand.lib,
                    subcommand.interpreter.into(),
                ),
                None => Package::new(folder_name, subcommand.lib, subcommand.interpreter.into()),
            };

            match package_manager.create_package(working_directory, &package) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    "Package created successfully.",
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
                &format!("Shell Package Manager (spm) version: {}", crate_version!()),
            );
        }
        Commands::Add(subcommand) => {
            // Check if we're in a package directory (has package.json)
            let current_dir: &Path = Path::new("./");
            if !package::is_inside_a_package(current_dir).unwrap_or(false) {
                display_message(
                    display_control::Level::Error,
                    "Not inside a package directory. Please run this command from a valid SPM package directory.",
                );
                return;
            }

            // Handle if the package is a remote repository.
            // Get a path to a dependency to install afterall.
            let mut is_move: bool = false;
            let mut temporary_path_opt: Option<PathBuf> = None;
            let (url_or_path, dependency_package_path) = handle_installation_path(
                &subcommand.path,
                &subcommand.base_url,
                &mut temporary_path_opt,
                &mut is_move,
            );

            match package_manager.add_dependency(
                current_dir,
                dependency_package_path.as_path(),
                &url_or_path,
                &subcommand.version,
            ) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    &format!(
                        "Successfully added dependency from {}",
                        dependency_package_path.display()
                    ),
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("Error adding dependency: {}", error.to_string()),
                ),
            }

            // Clean up the temporary directory if used
            if let Some(temporary_path) = temporary_path_opt {
                if let Err(cleanup_err) = cleanup_temporary_repository(&temporary_path) {
                    display_message(
                        display_control::Level::Warn,
                        &format!("Failed to clean up temporary directory: {}", cleanup_err),
                    );
                }
            }
        }
        Commands::Remove(subcommand) => {
            // Check if we're in a package directory (has package.json)
            let current_dir: &Path = Path::new("./");
            if !package::is_inside_a_package(current_dir).unwrap_or(false) {
                display_message(
                    display_control::Level::Error,
                    "Not inside a package directory. Please run this command from a valid SPM package directory.",
                );
                return;
            }

            // Get the namespace for display before we move it
            let (name, namespace) = match extract_name_and_namespace(&subcommand.name) {
                Ok((name, namespace)) => (name, namespace),
                Err(error) => {
                    display_message(
                        display_control::Level::Error,
                        &format!("Error extracting name and namespace: {}", error.to_string()),
                    );
                    return;
                }
            };
            let dependency_desc: String = format!("'{}/{}'", namespace, name);

            // Remove the dependency from the package
            match package_manager.remove_dependency(current_dir, &name, &namespace) {
                Ok(_) => display_message(
                    display_control::Level::Logging,
                    &format!("Successfully removed dependency {}", dependency_desc),
                ),
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("Error removing dependency: {}", error.to_string()),
                ),
            }
        }
        Commands::Refresh(subcommand) => {
            // Check if we're in a package directory (has package.json)
            let current_dir = Path::new("./");
            if !package::is_inside_a_package(current_dir).unwrap_or(false) {
                display_message(
                    display_control::Level::Error,
                    "Not inside a package directory. Please run this command from a valid SPM package directory.",
                );
                return;
            }

            // Refresh dependencies
            match package_manager.refresh_dependencies(
                current_dir,
                subcommand.version.as_deref(),
            ) {
                Ok(dependencies) => {
                    if dependencies.is_empty() {
                        display_message(
                            display_control::Level::Logging,
                            "No dependencies to refresh.",
                        );
                    } else {
                        let dep_message = if dependencies.len() == 1 {
                            format!(
                                "Successfully refreshed dependency: {}",
                                dependencies[0]
                            )
                        } else {
                            format!(
                                "Successfully refreshed {} dependencies: {}",
                                dependencies.len(),
                                dependencies.join(", ")
                            )
                        };
                        display_message(display_control::Level::Logging, &dep_message);
                    }
                }
                Err(error) => display_message(
                    display_control::Level::Error,
                    &format!("Error refreshing dependencies: {}", error.to_string()),
                ),
            }
        }
    }

    return;
}
