use std::path::{Path, PathBuf};

use anyhow::{Error, Result, anyhow};
use auth_git2::GitAuthenticator;
use git2::{Config, FetchOptions, ProxyOptions, RemoteCallbacks, Repository, build::RepoBuilder};

use super::utilities::create_temp_directory;

fn build_git_config<'a>(
    authenticator: &'a GitAuthenticator,
    git_config: &'a Config,
    fetch_options: &mut FetchOptions<'a>,
) -> Result<(), Error> {
    // Initialize git options
    let mut proxy_options: ProxyOptions<'a> = ProxyOptions::new();
    let mut remote_callbacks: RemoteCallbacks<'a> = RemoteCallbacks::new();

    // Set git up
    remote_callbacks.credentials(authenticator.credentials(git_config));
    proxy_options.auto();
    fetch_options.proxy_options(proxy_options);
    fetch_options.remote_callbacks(remote_callbacks);

    Ok(())
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
    let mut fetch_options: FetchOptions<'_> = FetchOptions::new();
    build_git_config(&auth, &git_config, &mut fetch_options)?;

    // Create a temp directory for the repository
    let temp_dir: PathBuf = create_temp_directory()?;
    let repo_temp_dir: PathBuf = temp_dir.join(repository);

    // Clone into the temporary directory
    let repository: Repository = RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(&clone_url, &repo_temp_dir)?;

    return Ok(repository.workdir().unwrap().to_path_buf());
}

/// Fetches a remote git repository with the specified version (tag or branch)
/// If destination is provided, clone directly to that location
/// If version is None, the default branch will be used
pub fn fetch_remote_git_repository_with_version(
    repository_url: &str,
    version: Option<&str>,
    destination: Option<&Path>,
) -> Result<PathBuf, Error> {
    // Initialize git configurations
    let auth: GitAuthenticator = GitAuthenticator::default();
    let git_config: Config = Config::open_default()?;

    // Initialize git options
    let mut fetch_options: FetchOptions<'_> = FetchOptions::new();
    build_git_config(&auth, &git_config, &mut fetch_options)?;

    // Determine the destination directory
    let repo_dir: PathBuf = if let Some(dest) = destination {
        dest.to_path_buf()
    } else {
        // Create a temp directory for the repository
        let temp_dir = create_temp_directory()?;

        // Extract repo name from URL
        let repo_name = extract_repo_name_from_url(repository_url);
        temp_dir.join(repo_name)
    };

    // Create the parent directory if it doesn't exist
    if let Some(parent) = repo_dir.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Clone the repository
    let mut builder: RepoBuilder<'_> = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    // Clone the repository
    let repo: Repository = builder.clone(repository_url, &repo_dir)?;

    // Checkout the specified version if provided
    if let Some(ver) = version {
        // Try to find a reference matching the version (tag or branch)
        let mut checkout_revision = None;

        // Check if it's a tag
        if let Ok(reference) = repo.find_reference(&format!("refs/tags/{}", ver)) {
            checkout_revision = reference.target();
        }

        // If not found as a tag, check if it's a branch
        if checkout_revision.is_none() {
            if let Ok(reference) = repo.find_reference(&format!("refs/remotes/origin/{}", ver)) {
                checkout_revision = reference.target();
            }
        }

        // If we found a reference, check it out
        if let Some(oid) = checkout_revision {
            // Get the commit for this revision
            let commit = repo.find_commit(oid)?;

            // Create and checkout the branch
            let mut checkout_opts = git2::build::CheckoutBuilder::new();
            checkout_opts.force();

            // Check if there's already a local branch with this name
            let branch_exists = repo.find_branch(ver, git2::BranchType::Local).is_ok();

            if !branch_exists {
                // Create a new local branch pointing to the commit
                repo.branch(ver, &commit, false)?;
            }

            // Get the reference to the local branch
            let reference = repo.find_reference(&format!("refs/heads/{}", ver))?;

            // Set HEAD to this reference
            repo.set_head(
                reference
                    .name()
                    .ok_or_else(|| anyhow!("Invalid reference name"))?,
            )?;

            // Checkout the working directory
            repo.checkout_head(Some(&mut checkout_opts))?;
        } else {
            return Err(anyhow!("Version '{}' not found in repository", ver));
        }
    }

    Ok(repo.workdir().unwrap_or(&repo_dir).to_path_buf())
}

/// Extracts repository name from a URL
fn extract_repo_name_from_url(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.is_empty() {
        return "repo".to_string(); // Fallback
    }

    let last_part = parts.last().unwrap_or(&"repo");
    last_part.trim_end_matches(".git").to_string()
}

pub fn is_git_repository_link(expression: &str) -> bool {
    !Path::new(expression).exists()
}
