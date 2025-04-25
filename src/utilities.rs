use std::path::{Path, PathBuf};

use anyhow::{Error, Result, anyhow};
use auth_git2::GitAuthenticator;
use git2::{Config, FetchOptions, ProxyOptions, RemoteCallbacks, Repository, build::RepoBuilder};

use crate::{
    display_control::{Level, display_form, display_message, display_tree_message, input_message},
    package::{Package, PackageManager, PackageMetadata, is_inside_a_package},
    shell::execute_shell_script,
    properties::{DEFAULT_SPM_FOLDER, DEFAULT_TEMPORARY_FOLDER}
};

// Create the temporary directory for cloning remote repositories
pub fn create_temp_directory() -> Result<PathBuf, Error> {
    let temp_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to locate home directory"))?
        .join("DEFAULT_SPM_FOLDER")
        .join("temp");

    // Create the temp directory if it doesn't exist
    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)?;
    }

    Ok(temp_dir)
}

// Clean up the temporary directory for a specific repository
pub fn cleanup_temp_repository(repo_path: &Path) -> Result<(), Error> {
    if repo_path.exists()
        && repo_path.starts_with(dirs::home_dir().unwrap().join(DEFAULT_SPM_FOLDER).join(DEFAULT_TEMPORARY_FOLDER))
    {
        std::fs::remove_dir_all(repo_path)?;
    }

    Ok(())
}

pub fn execute_run_command(
    package_manager: &PackageManager,
    expression: String,
    args: &[String],
) -> Result<(), Error> {
    let path: &Path = Path::new(&expression);

    // Case 1: input is a shell script
    if path.is_file() {
        return execute_shell_script(&expression, args);
    }

    // Case 2: input is a shell script project/package
    if path.is_dir() {
        // Validate the directory
        if is_inside_a_package(path)? {
            let package = Package::from_file(path)?;
            let main_entrypoint_filename: &str = package.access_main_entrypoint();

            return execute_shell_script(
                &path
                    .join(main_entrypoint_filename)
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                args,
            );
        }
    }

    // Case 3: Check if it's an installed package name first
    // Try to find exact package name match (ignoring namespace)
    let package_candidates: Vec<PackageMetadata> = package_manager.keyword_search(&expression)?;

    if !package_candidates.is_empty() {
        // Run the package if it is exactly one match
        if package_candidates.len() == 1 {
            let package_metadata = &package_candidates[0];
            display_message(
                Level::Logging,
                &format!("Running package: {}", package_metadata.get_full_name()),
            );
            return execute_shell_script(package_metadata.get_main_entry_point(), args);
        }

        // If multiple matches, let user choose
        display_message(Level::Logging, "Multiple packages found:");
        for (index, package_metadata) in package_candidates.iter().enumerate() {
            display_tree_message(
                1,
                &format!("{}: {}", index + 1, package_metadata.get_full_name()),
            );
        }
        let selection: usize = input_message("Please select a package to execute:")?
            .trim()
            .parse::<usize>()?;

        if selection < 1 || selection > package_candidates.len() {
            return Err(anyhow!("Invalid selection"));
        }

        let selected_package = &package_candidates[selection - 1];
        display_message(
            Level::Logging,
            &format!("Running package: {}", selected_package.get_full_name()),
        );

        return execute_shell_script(selected_package.get_main_entry_point(), args);
    }

    // Case 4: Input appears to be a GitHub repository URL
    if expression.starts_with("http://")
        || expression.starts_with("https://")
        || is_git_repository_link(&expression)
    {
        display_message(
            Level::Logging,
            &format!("Fetching package from remote repository: {}", expression),
        );

        // Default to GitHub if no specific domain is specified
        let base_url = if expression.contains("github.com") || !expression.contains("://") {
            "https://github.com"
        } else {
            // Extract base URL from the full URL
            let parts: Vec<&str> = expression.split("/").collect();
            if parts.len() >= 3 {
                &format!("{}//{}", parts[0], parts[2])
            } else {
                "https://github.com"
            }
        };

        // Extract repository path from the URL
        let repo_path = if expression.contains("github.com") {
            let parts: Vec<&str> = expression.split("github.com/").collect();
            if parts.len() > 1 {
                parts[1]
            } else {
                &expression
            }
        } else if expression.contains("://") {
            let parts: Vec<&str> = expression.split("/").collect();
            if parts.len() >= 4 {
                &expression[parts[0].len() + parts[1].len() + parts[2].len() + 3..]
            } else {
                &expression
            }
        } else {
            &expression
        };

        // Fetch the repository to a temporary directory
        let temp_dir = match fetch_remote_git_repository(base_url, repo_path) {
            Ok(dir) => dir,
            Err(e) => return Err(anyhow!("Failed to fetch remote repository: {}", e)),
        };

        // Validate the fetched repository
        if is_inside_a_package(&temp_dir)? {
            let package = Package::from_file(&temp_dir)?;
            let main_entrypoint_filename: &str = package.access_main_entrypoint();

            display_message(
                Level::Logging,
                &format!("Running package: {}", package.get_full_name()),
            );

            // Run the script
            let result = execute_shell_script(
                &temp_dir
                    .join(main_entrypoint_filename)
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                args,
            );

            // Clean up the temporary directory
            if let Err(cleanup_err) = cleanup_temp_repository(&temp_dir) {
                display_message(
                    Level::Warn,
                    &format!("Failed to clean up temporary directory: {}", cleanup_err),
                );
            }

            return result;
        } else {
            // Clean up even on failure
            if let Err(cleanup_err) = cleanup_temp_repository(&temp_dir) {
                display_message(
                    Level::Warn,
                    &format!("Failed to clean up temporary directory: {}", cleanup_err),
                );
            }

            return Err(anyhow!(
                "The fetched repository does not contain a valid package"
            ));
        }
    }

    // If we get here, no packages were found
    return Err(anyhow!("No packages found with name: {}", expression));
}

pub fn show_packages(packages_metadata: &Vec<PackageMetadata>) {
    let mut form_data: Vec<Vec<String>> = Vec::new();

    for (index, metadata) in packages_metadata.iter().enumerate() {
        form_data.push(vec![
            index.to_string(),
            metadata.get_full_name(),
            metadata.get_description().to_string(),
            metadata.get_version().to_string(),
        ]);
    }

    display_form(vec!["Index", "Name", "Description", "Version"], &form_data);
}

pub fn fetch_remote_git_repository(base_url: &str, repository: &str) -> Result<PathBuf, Error> {
    let mut clone_url: String = String::new();
    if !base_url.ends_with("/") {
        clone_url.push_str(&format!("{}/{}", base_url, repository));
    } else {
        clone_url.push_str(&format!("{}{}", base_url, repository));
    }

    // Initialize git configurations
    let auth: GitAuthenticator = GitAuthenticator::default();
    let git_config: Config = Config::open_default()?;

    // Initialize git options
    let mut fetch_options = FetchOptions::new();
    let mut proxy_options = ProxyOptions::new();
    let mut remote_callbacks = RemoteCallbacks::new();

    // Set git up
    remote_callbacks.credentials(auth.credentials(&git_config));
    proxy_options.auto();
    fetch_options.proxy_options(proxy_options);
    fetch_options.remote_callbacks(remote_callbacks);

    // Create a temp directory for the repository
    let temp_dir = create_temp_directory()?;
    let repo_temp_dir = temp_dir.join(repository);

    // Clone into the temporary directory
    let repository: Repository = RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(&clone_url, &repo_temp_dir)?;

    return Ok(repository.workdir().unwrap().to_path_buf());
}

pub fn is_git_repository_link(expression: &str) -> bool {
    !Path::new(expression).exists()
}

/// Checks if a given directory is in the user's PATH environment variable.
///
/// This function compares the provided directory path with each directory in the PATH,
/// accounting for possible path normalization and canonicalization.
///
/// # Arguments
///
/// * `dir` - A reference to the path to check for in the PATH.
///
/// # Returns
///
/// A boolean that is `true` if the directory is in the PATH, or `false` otherwise.
pub fn is_directory_in_path(dir: &Path) -> bool {
    // Get the PATH environment variable
    let path = match std::env::var("PATH") {
        Ok(p) => p,
        Err(_) => return false, // If PATH isn't defined, return false
    };

    // Canonicalize the input directory if possible
    let canonical_dir = match dir.canonicalize() {
        Ok(d) => d,
        Err(_) => return false, // If the directory doesn't exist, return false
    };

    // Split the PATH by the platform-specific path separator and check each directory
    for path_dir in std::env::split_paths(&path) {
        // Try to canonicalize the path directory
        if let Ok(canonical_path_dir) = path_dir.canonicalize() {
            if canonical_path_dir == canonical_dir {
                return true;
            }
        }
    }

    false
}

/// Checks if the binary directory is in the PATH and sets it up automatically if not.
/// This function automatically adds the SPM bin directory to the user's PATH during first run.
///
/// # Arguments
///
/// * `package_manager` - A reference to the PackageManager to check its binary directory.
pub fn check_bin_directory_in_path(package_manager: &PackageManager) {
    if let Ok(bin_dir) = package_manager.get_bin_directory() {
        if !is_directory_in_path(&bin_dir) {
            let path_str = bin_dir.to_string_lossy();

            // Setting up automatically on first run
            display_message(
                Level::Logging,
                &format!(
                    "Setting up SPM environment: adding '{}' to your PATH.",
                    path_str
                ),
            );

            match setup_environment_for_user(&bin_dir) {
                Ok(_) => {
                    display_message(
                        Level::Logging,
                        &format!(
                            "Successfully added '{}' to your PATH. You may need to restart your terminal or run 'source ~/.bashrc' (or your shell's equivalent) for changes to take effect.",
                            path_str
                        ),
                    );
                }
                Err(e) => {
                    display_message(
                        Level::Error,
                        &format!("Failed to set up environment: {}", e),
                    );
                    display_message(
                        Level::Warn,
                        &format!(
                            "Please manually add '{}' to your PATH to use SPM commands.",
                            path_str
                        ),
                    );

                    // Show manual setup instructions
                    if cfg!(target_os = "windows") {
                        display_message(
                            Level::Warn,
                            "To add it to your PATH, update your Environment Variables through System Properties.",
                        );
                    } else {
                        display_message(
                            Level::Warn,
                            &format!(
                                "Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):\nexport PATH=\"{}:$PATH\"",
                                path_str
                            ),
                        );
                    }
                }
            }
        }
    }
}

/// Sets up the environment for the user by adding the SPM bin directory to their PATH.
///
/// # Arguments
///
/// * `bin_dir` - A reference to the path of the bin directory to add to PATH.
///
/// # Returns
///
/// A `Result` indicating success or failure.
fn setup_environment_for_user(bin_dir: &Path) -> Result<(), Error> {
    let path_str = bin_dir.to_string_lossy();

    // Determine which shell configuration file to modify
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    // Try to identify the user's shell
    let shell_var = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));

    let config_file = if shell_var.contains("zsh") {
        home_dir.join(".zshrc")
    } else if shell_var.contains("fish") {
        home_dir.join(".config/fish/config.fish")
    } else {
        // Default to bash
        home_dir.join(".bashrc")
    };

    // Create the configuration file if it doesn't exist
    if !config_file.exists() {
        std::fs::File::create(&config_file)?;
    }

    // Read the existing content
    let content: String = std::fs::read_to_string(&config_file)?;

    // Check if the PATH export already exists
    let export_line = if shell_var.contains("fish") {
        format!("set -gx PATH \"{}\" $PATH", path_str)
    } else {
        format!("export PATH=\"{}:$PATH\"", path_str)
    };

    if !content.contains(&export_line) {
        // Append the export line to the end of the file
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&config_file)?;

        use std::io::Write;

        // Add a newline if the file doesn't end with one
        if !content.ends_with('\n') && !content.is_empty() {
            writeln!(file)?;
        }

        // Add a comment explaining what this is for
        writeln!(file, "\n# Added by Shell Package Manager (SPM)")?;
        writeln!(file, "{}", export_line)?;

        // For fish shell, we might need to do something different
        if shell_var.contains("fish") {
            // Execute the command to make it take effect in the current session
            std::process::Command::new("fish")
                .arg("-c")
                .arg(&export_line)
                .output()
                .ok(); // Ignore errors here
        }
    }

    Ok(())
}
