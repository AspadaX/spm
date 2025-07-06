use std::{
    path::{Path, PathBuf},
};

use anyhow::{Error, Result, anyhow};
use auth_git2::GitAuthenticator;
use git2::{Config, FetchOptions, ProxyOptions, RemoteCallbacks, build::RepoBuilder};

use crate::{
    display_control::{display_form, display_message, display_tree_message, input_message, Level},
    program::{ProgramManager, Program},
    properties::{DEFAULT_SPM_FOLDER, DEFAULT_TEMPORARY_FOLDER},
    shell::{execute_shell_script_with_context, ExecutionContext},
};

// Create the temporary directory for cloning remote repositories
pub fn create_temp_directory() -> Result<PathBuf, Error> {
    let temp_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to locate home directory"))?
        .join(DEFAULT_SPM_FOLDER)
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
    program_manager: &ProgramManager,
    expression: String,
    args: &[String],
) -> Result<(), Error> {
    let path: &Path = Path::new(&expression);

    // Case 1: input is a shell script file
    if path.is_file() {
        // Execute regular shell script in the current working directory
        return execute_shell_script_with_context(
            &expression,
            args,
            ExecutionContext::CurrentWorkingDirectory,
        );
    }

    // Case 2: Check if it's an installed program name
    let program_candidates: Vec<Program> = program_manager.keyword_search(&expression)?;

    if !program_candidates.is_empty() {
        // Run the program if it is exactly one match
        if program_candidates.len() == 1 {
            let program = &program_candidates[0];
            display_message(
                Level::Logging,
                &format!("Running program: {}", program.get_name()),
            );
            // Execute from current working directory when using spm run
            return execute_shell_script_with_context(
                program.get_program_path().ok_or_else(|| anyhow!("Program path not available"))?,
                args,
                ExecutionContext::CurrentWorkingDirectory,
            );
        }

        // If multiple matches, let user choose
        display_message(Level::Logging, "Multiple programs found:");
        for (index, program) in program_candidates.iter().enumerate() {
            display_tree_message(
                1,
                &format!("{}: {}", index + 1, program.get_name()),
            );
        }
        let selection: usize = input_message("Please select a program to execute:")?
            .trim()
            .parse::<usize>()?;

        if selection < 1 || selection > program_candidates.len() {
            return Err(anyhow!("Invalid selection"));
        }

        let selected_program = &program_candidates[selection - 1];
        display_message(
            Level::Logging,
            &format!("Running program: {}", selected_program.get_name()),
        );

        // Execute from current working directory when using spm run
        return execute_shell_script_with_context(
            selected_program.get_program_path().ok_or_else(|| anyhow!("Program path not available"))?,
            args,
            ExecutionContext::CurrentWorkingDirectory,
        );
    }

    // If we get here, no programs were found
    return Err(anyhow!("No programs found with name: {}", expression));
}

pub fn show_programs(programs: &Vec<Program>) {
    let mut form_data: Vec<Vec<String>> = Vec::new();

    for (index, program) in programs.iter().enumerate() {
        form_data.push(vec![
            index.to_string(),
            program.get_name().to_string(),
            program.get_interpreter().to_string(),
            program.get_program_path().unwrap_or("N/A").to_string(),
        ]);
    }

    display_form(vec!["Index", "Name", "Interpreter", "Path"], &form_data);
}

pub fn clone_git_repository(git_url: &str, destination: &Path) -> Result<(), Error> {
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

    // Clone into the destination directory
    RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(git_url, destination)?;

    Ok(())
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

pub fn check_bin_directory_in_path() -> Result<bool, Error> {
    let program_manager = ProgramManager::new()?;
    let bin_directory = program_manager.get_bin_directory()?;

    Ok(is_directory_in_path(&bin_directory))
}
