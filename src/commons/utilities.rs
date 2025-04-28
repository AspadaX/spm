use std::fs::DirEntry;
use std::path::{Path, PathBuf};

use anyhow::{Error, Result, anyhow};

use crate::properties::{DEFAULT_DEPENDENCIES_FOLDER, DEFAULT_LOCAL_PACKAGE_NAMESPACE};
use crate::{
    display_control::{Level, display_form, display_message, display_tree_message, input_message},
    package::{Package, PackageManager, PackageMetadata, is_inside_a_package},
    properties::{DEFAULT_SPM_FOLDER, DEFAULT_TEMPORARY_FOLDER},
    shell::{ExecutionContext, execute_shell_script_with_context},
};

use super::git::{fetch_remote_git_repository, is_git_repository_link};

/// Recursively copies the contents of a directory
///
/// # Arguments
/// * `src` - The source directory to copy from
/// * `dst` - The destination directory to copy to
///
/// # Example
/// ```
/// use std::path::Path;
/// use crate::utilities::copy_dir_all;
/// copy_dir_all(Path::new("./src"), Path::new("./dst")).unwrap();
/// ```
pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry: DirEntry = entry?;
        let entry_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        if entry_path.is_dir() {
            copy_dir_all(&entry_path, &dest_path)?;
        } else {
            std::fs::copy(&entry_path, &dest_path)?;
        }
    }
    Ok(())
}

/// Creates (if necessary) and returns the path to the temporary directory used for cloning remote repositories.
///
/// This function constructs a path in the user's home directory under the default SPM folder and a "temp" subdirectory.
/// If the directory does not exist, it will be created. The resulting path is returned as a `PathBuf`.
///
/// # Returns
///
/// * `Ok(PathBuf)` - The path to the temporary directory.
/// * `Err(Error)` - If the home directory cannot be determined or the directory cannot be created.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use crate::utilities::create_temp_directory;
///
/// let temp_dir: PathBuf = create_temp_directory().expect("Failed to create temp directory");
/// assert!(temp_dir.exists());
/// ```
///
pub fn create_temporary_directory() -> Result<PathBuf, Error> {
    let temporary_dir: PathBuf = dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to locate home directory"))?
        .join(DEFAULT_SPM_FOLDER)
        .join(DEFAULT_TEMPORARY_FOLDER);

    // Create the temporary directory if it doesn't exist
    if !temporary_dir.exists() {
        std::fs::create_dir_all(&temporary_dir)?;
    }

    Ok(temporary_dir)
}

// Clean up the temporary directory for a specific repository
pub fn cleanup_temporary_repository(repo_path: &Path) -> Result<(), Error> {
    if repo_path.exists()
        && repo_path.starts_with(
            dirs::home_dir()
                .unwrap()
                .join(DEFAULT_SPM_FOLDER)
                .join(DEFAULT_TEMPORARY_FOLDER),
        )
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
        // Execute regular shell script in the current working directory
        return execute_shell_script_with_context(
            &expression,
            args,
            ExecutionContext::CurrentWorkingDirectory,
        );
    }

    // Case 2: input is a shell script project/package
    if path.is_dir() {
        // Validate the directory
        if is_inside_a_package(path)? {
            let package = Package::from_file(path)?;
            let main_entrypoint_filename: &str = package.access_main_entrypoint();
            // Execute from the current working directory for local package run
            return execute_shell_script_with_context(
                &path
                    .join(main_entrypoint_filename)
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                args,
                ExecutionContext::CurrentWorkingDirectory,
            );
        }
    }

    // Case 3: Check if it's an installed package name first
    let package_candidates: Vec<PackageMetadata> = package_manager.keyword_search(&expression)?;

    if !package_candidates.is_empty() {
        // Run the package if it is exactly one match
        if package_candidates.len() == 1 {
            let package_metadata = &package_candidates[0];
            display_message(
                Level::Logging,
                &format!("Running package: {}", package_metadata.get_full_name()),
            );
            // Execute from current working directory when using spm run
            return execute_shell_script_with_context(
                package_metadata.get_main_entry_point(),
                args,
                ExecutionContext::CurrentWorkingDirectory,
            );
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

        // Execute from current working directory when using spm run
        return execute_shell_script_with_context(
            selected_package.get_main_entry_point(),
            args,
            ExecutionContext::CurrentWorkingDirectory,
        );
    }

    // If we get here, no packages were found
    Err(anyhow!("No packages found with name: {}", expression))
}

/// Display packages with a column sytle
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

/// Handles the installation path for a package, determining whether it is a local path or a remote git repository.
///
/// This function checks if the provided `path` is a git repository link or a local directory. If it is a git repository,
/// it fetches the repository to a temporary directory and sets the `is_move` flag to `true`. If it is a local path,
/// it simply returns the path as a `PathBuf`.
///
/// # Arguments
///
/// * `path` - The path or repository identifier (e.g., "username/repo" or a local directory path).
/// * `base_url` - The base URL to use for fetching remote git repositories.
/// * `temp_path_opt` - A mutable reference to an `Option<PathBuf>` to store the temporary path for later cleanup if needed.
/// * `is_move` - A mutable reference to a boolean flag indicating whether the package should be moved after installation.
///
/// # Returns
///
/// Returns a `PathBuf` representing the resolved package path (either local or the path to the cloned repository).
///
/// # Examples
///
/// ```
/// // Example 1: Installing from a remote git repository
/// let mut temp_path: Option<std::path::PathBuf> = None;
/// let mut is_move = false;
/// let base_url = "https://github.com";
/// let repo_path = handle_installation_path("username/repo", base_url, &mut temp_path, &mut is_move);
/// assert!(repo_path.exists());
/// assert!(is_move);
///
/// // Example 2: Installing from a local directory
/// let mut temp_path: Option<std::path::PathBuf> = None;
/// let mut is_move = false;
/// let local_path = handle_installation_path("./my-local-package", "", &mut temp_path, &mut is_move);
/// assert_eq!(local_path, std::path::PathBuf::from("./my-local-package"));
/// assert!(!is_move);
/// ```
pub fn handle_installation_path(
    path: &str,
    base_url: &str,
    temporary_path_opt: &mut Option<PathBuf>,
    is_move: &mut bool,
) -> (String, PathBuf) {
    // This is an url if it is a git repository, or a local path,
    // if it is a local path.
    let string_representation: String;
    let package_path: PathBuf;

    // Determine whether this is a remote installation, or local
    if is_git_repository_link(path) {
        // Create a subcommand for handling git repository installations
        let cmd_parts: Vec<&str> = path.split("/").collect();
        if cmd_parts.len() < 2 {
            display_message(
                Level::Error,
                "Invalid Git repository format. Expected: username/repo",
            );
            return ("".to_string(), PathBuf::new());
        }

        // Fetch the repository to a temporary directory
        package_path = match fetch_remote_git_repository(base_url, &path) {
            Ok(result) => {
                // Store the temporary path for later cleanup
                *temporary_path_opt = Some(result.clone());
                string_representation = format!("{}/{}", base_url, path);
                result
            }
            Err(error) => {
                display_message(Level::Error, &format!("{}", error.to_string()));
                return ("".to_string(), PathBuf::new());
            }
        };

        // Move the local git repository for installations
        *is_move = true;
    } else {
        string_representation = path.to_string();
        package_path = Path::new(path).to_path_buf();
    }

    (string_representation, package_path)
}

/// Extract the name and namespace from a repo url, a local path, or a short expression
/// like `some-namespace/some-package`
pub fn extract_name_and_namespace(text: &str) -> Result<(String, String), Error> {
    // 1) If it’s a local path, just use the last directory name
    if Path::new(text).exists() {
        if let Some(os_name) = Path::new(text).file_name() {
            let local_name: String = os_name.to_string_lossy().to_string();
            // Name, Namespace
            return Ok((local_name, DEFAULT_LOCAL_PACKAGE_NAMESPACE.to_string()));
        }
    }

    // 2) Otherwise treat it like a remote URL and do the current logic
    // Extract repo name from URL (e.g., https://github.com/username/repo-name)
    let mut parts: Vec<&str> = text.split('/').collect();
    if parts.len() > 2 {
        let repo_name: String = parts.last().unwrap_or(&"").to_string();
        let username: String = parts[parts.len() - 2].to_string();
        let name: String = repo_name.trim_end_matches(".git").to_string();
        // Name, Namespace
        return Ok((name, username));
    }

    // If the input is `some-namespace/some-package`
    if parts.len() == 2 {
        let results: Vec<String> = parts.iter_mut().map(|item| item.to_string()).collect();
        // Name, Namespace
        return Ok((results[1].clone(), results[0].clone()));
    }

    return Err(anyhow!("Wrong input"));
}

/// Construct the folder name used under the "dependencies" directory. 
/// Return error when a dependency folder does not exist in the package,
/// or if the given dependency package path does not eixst.
pub fn construct_dependency_path(
    package_path: &Path,
    namespace: &str,
    name: &str,
) -> Result<PathBuf, Error> {
    let root_dependencies_directory: PathBuf = package_path.join(DEFAULT_DEPENDENCIES_FOLDER);

    if !root_dependencies_directory.exists() {
        return Err(anyhow!(
            "The project lacks a folder for dependency. Please check the project integrity"
        ));
    }

    let dependencies_directory: PathBuf = root_dependencies_directory.join(namespace).join(name);

    if !dependencies_directory.exists() {
        return Err(anyhow!(
            "{} does not exist. Please check the dependency integrity",
            package_path.display()
        ));
    }

    Ok(dependencies_directory)
}
