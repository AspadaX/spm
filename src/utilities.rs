use std::path::{Path, PathBuf};

use anyhow::{Error, Result, anyhow};
use auth_git2::GitAuthenticator;
use git2::{build::RepoBuilder, Config, FetchOptions, ProxyOptions, RemoteCallbacks, Repository};

use crate::{
    display_control::{Level, display_form, display_message, display_tree_message, input_message},
    package::{Package, PackageManager, PackageMetadata, is_inside_a_package},
    shell::execute_shell_script,
};

pub fn execute_run_command(
    package_manager: &PackageManager,
    expression: String,
) -> Result<(), Error> {
    let path: &Path = Path::new(&expression);

    // Case 1: input is a shell script
    if path.is_file() {
        return execute_shell_script(&expression);
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
            );
        }
    } 
    
    // Case 3: Input is a keyword or keywords
    let package_candidates: Vec<PackageMetadata> = package_manager.keyword_search(&expression)?;
    // Throw an error if no chains are found
    if package_candidates.len() == 0 {
        return Err(anyhow!("No packages found"));
    }

    // Run the chain if it is exactly one
    if package_candidates.len() == 1 {
        return execute_shell_script(
            &path
                .join(package_candidates[0].get_main_entry_point())
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );
    }

    display_message(Level::Logging, "Multiple packages found:");
    for (index, package_metadata) in package_candidates.iter().enumerate() {
        display_tree_message(
            1,
            &format!("{}: {}", index + 1, package_metadata.get_pacakge_name()),
        );
    }
    let selection: usize = input_message("Please select a chain to execute:")?
        .trim()
        .parse::<usize>()?;

    return execute_shell_script(
        &path
            .join(package_candidates[selection - 1].get_main_entry_point())
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
}

pub fn show_packages(packages_metadata: &Vec<PackageMetadata>) {
    let mut form_data: Vec<Vec<String>> = Vec::new();

    for (index, metadata) in packages_metadata.iter().enumerate() {
        form_data.push(vec![
            index.to_string(),
            metadata.get_pacakge_name().to_string(),
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
    
    let current_dir: PathBuf = std::env::current_dir()?
        .canonicalize()?
        .join(repository.split("/").last().unwrap()); // Namespace for pacakges?
    
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
    
    // Clone
    let repository: Repository = RepoBuilder::new()
        .fetch_options(
            fetch_options
        )
        .clone(&clone_url, &current_dir)?;
    
    return Ok(repository.workdir().unwrap().to_path_buf());
}

pub fn is_git_repository_link(expression: &str) -> bool {
    !Path::new(expression).exists()
}