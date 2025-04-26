use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

use crate::{
    commons::{git::fetch_remote_git_repository_with_version, utilities::copy_dir_all},
    properties::{
        DEFAULT_DEPENDENCIES_FOLDER, DEFAULT_LOCAL_PACKAGE_NAMESPACE, DEFAULT_PACKAGE_JSON,
    },
};

use super::Package;

/// Represents a single dependency with repository URL and version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    /// The full repository URL where the package can be downloaded
    pub url: String,
    /// The version of the package (using semver, branch name, or commit hash)
    pub version: String,
    /// Name of the dependency (derived from URL)
    #[serde(skip)]
    pub name: String,
    /// Namespace of the dependency (derived from URL, optional)
    #[serde(skip)]
    pub namespace: Option<String>,
}

impl Dependency {
    /// Creates a new dependency with the specified repository URL and version
    pub fn new(url: String, version: String) -> Self {
        let (name, namespace) = Self::extract_name_and_namespace(&url);
        Self {
            url,
            version,
            name,
            namespace,
        }
    }

    /// Extracts the name and namespace from a repository URL
    fn extract_name_and_namespace(url: &str) -> (String, Option<String>) {
        // 1) If it’s a local path, just use the last directory name
        if Path::new(url).exists() {
            if let Some(os_name) = Path::new(url).file_name() {
                let local_name: String = os_name.to_string_lossy().to_string();
                return (
                    local_name,
                    Some(DEFAULT_LOCAL_PACKAGE_NAMESPACE.to_string()),
                );
            } else {
                // If there is no last component (e.g. "." or "/"), fallback
                return (
                    url.to_string(),
                    Some(DEFAULT_LOCAL_PACKAGE_NAMESPACE.to_string()),
                );
            }
        }

        // 2) Otherwise treat it like a remote URL and do the current logic
        // Extract repo name from URL (e.g., https://github.com/username/repo-name)
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let repo_name: String = parts.last().unwrap_or(&"").to_string();
            let username: String = parts[parts.len() - 2].to_string();

            let name: String = repo_name.trim_end_matches(".git").to_string();
            let namespace: Option<String> = if !username.is_empty() {
                Some(username)
            } else {
                None
            };
            (name, namespace)
        } else {
            // If URL format is unexpected, just treat entire string as name
            (url.to_string(), None)
        }
    }

    /// Gets the fully qualified name including namespace if available
    pub fn get_full_name(&self) -> String {
        match &self.namespace {
            Some(namespace) => format!("{}/{}", namespace, self.name),
            None => self.name.clone(),
        }
    }
}

/// Collection of dependencies as a vector
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependencies(Vec<Dependency>);

impl Dependencies {
    /// Creates a new empty dependencies collection
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Adds a dependency to the collection
    pub fn add(&mut self, dependency: Dependency) {
        // Only add if no dependency with the same URL and version exists
        if self
            .0
            .iter()
            .any(|d| d.url == dependency.url && d.version == dependency.version)
        {
            // Do not add duplicate
            return;
        }
        // If a dependency with the same name and namespace exists, replace it
        if let Some(index) =
            self.find_by_name_and_namespace(&dependency.name, &dependency.namespace)
        {
            self.0[index] = dependency;
        } else {
            self.0.push(dependency);
        }
    }

    /// Removes a dependency by name and namespace
    pub fn remove(&mut self, name: &str, namespace: &Option<String>) -> Option<Dependency> {
        if let Some(index) = self.find_by_name_and_namespace(name, namespace) {
            Some(self.0.remove(index))
        } else {
            None
        }
    }

    /// Finds a dependency by name and namespace
    pub fn find_by_name_and_namespace(
        &self,
        name: &str,
        namespace: &Option<String>,
    ) -> Option<usize> {
        self.0.iter().position(|d| {
            d.name == name
                && match (namespace, &d.namespace) {
                    (Some(n1), Some(n2)) => n1 == n2,
                    (None, None) => true,
                    _ => false,
                }
        })
    }

    /// Gets a reference to a dependency by name and namespace
    pub fn get_by_name_and_namespace(
        &self,
        name: &str,
        namespace: &Option<String>,
    ) -> Option<&Dependency> {
        self.find_by_name_and_namespace(name, namespace)
            .map(|idx| &self.0[idx])
    }

    /// Returns all dependencies
    pub fn get_all(&self) -> &[Dependency] {
        &self.0
    }

    /// Returns the number of dependencies
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if there are no dependencies
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    // Provide a public method returning a mutable reference to the inner Vec
    pub fn get_all_mut(&mut self) -> &mut Vec<Dependency> {
        &mut self.0
    }

    pub fn update(
        &mut self,
        package_path: &Path,
        name: &str,
        namespace: &Option<String>,
        version: Option<&str>,
    ) -> Result<Dependency, Error> {
        // Find the dependency to update
        let idx: usize = match self.find_by_name_and_namespace(name, namespace) {
            Some(idx) => idx,
            None => {
                return Err(anyhow!(
                    "Dependency '{}' with namespace '{:?}' not found in package.json",
                    name,
                    namespace
                ));
            }
        };

        // Get the dependency to be updated
        let dependency: &Dependency = &self.0[idx];

        let dependency_dir_name: String = match &dependency.namespace {
            Some(ns) => format!("{}-{}", ns, dependency.name),
            None => dependency.name.clone(),
        };
        let dependency_path: std::path::PathBuf = package_path
            .join(DEFAULT_DEPENDENCIES_FOLDER)
            .join(&dependency_dir_name);

        // Bail out if the dependency directory does not exist
        if !dependency_path.exists() {
            return Err(anyhow!(
                "{} does not exist in the {} folder, please double check if the package is intact",
                dependency.get_full_name(),
                DEFAULT_DEPENDENCIES_FOLDER
            ));
        }

        // Remove the existing dependency package
        std::fs::remove_dir_all(&dependency_path)?;

        // Construct the new dependency package
        let new_version: String = version.unwrap_or(&dependency.version).to_string();
        let updated_dependency: Dependency = Dependency::new(dependency.url.clone(), new_version);

        // --- Fetch/copy the dependency ---
        let is_local: bool = Path::new(&dependency.url).exists();
        if is_local {
            // Local path
            let dep_package_json_path: std::path::PathBuf =
                Path::new(&dependency.url).join(DEFAULT_PACKAGE_JSON);

            if !dep_package_json_path.exists() {
                return Err(anyhow!(
                    "The local path '{}' is not a valid SPM package (missing {})",
                    dependency.url,
                    DEFAULT_PACKAGE_JSON
                ));
            }

            let dep_package: Package = Package::from_file(Path::new(&dependency.url))?;
            if !dep_package.is_library {
                return Err(anyhow!(
                    "The package '{}' is not marked as a library. Only libraries can be added as dependencies.",
                    dep_package.get_full_name()
                ));
            }

            copy_dir_all(Path::new(&dependency.url), &dependency_path)?;
        }

        if !is_local {
            // Remote git repository
            match fetch_remote_git_repository_with_version(
                &updated_dependency.url,
                Some(&updated_dependency.version),
                Some(&dependency_path),
            ) {
                Ok(_) => {}
                Err(e) => return Err(anyhow!("Failed to update dependency: {}", e)),
            }
        }

        // --- Shared validation logic ---
        let dep_package_json_path = dependency_path.join("package.json");
        if !dep_package_json_path.exists() {
            std::fs::remove_dir_all(&dependency_path)?;
            return Err(anyhow!(
                "The updated dependency at '{}' is not a valid SPM package (missing package.json)",
                updated_dependency.url
            ));
        }
        let dep_package = super::metadata::Package::from_file(&dependency_path)?;
        if !dep_package.is_library {
            std::fs::remove_dir_all(&dependency_path)?;
            return Err(anyhow!(
                "The updated package '{}' is not marked as a library. Only libraries can be added as dependencies.",
                dep_package.get_full_name()
            ));
        }

        // --- Update the dependency in the collection ---
        self.0[idx] = updated_dependency.clone();

        // --- Update package.json ---
        let package_json_path = package_path.join("package.json");
        let mut package = super::metadata::Package::from_file(package_path)?;
        package.dependencies = self.clone();
        let file: File = File::create(&package_json_path)?;
        serde_json::to_writer_pretty(file, &package)?;

        Ok(updated_dependency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new(
            "https://github.com/user/repo".to_string(),
            "1.0.0".to_string(),
        );
        assert_eq!(dep.url, "https://github.com/user/repo");
        assert_eq!(dep.version, "1.0.0");
        assert_eq!(dep.name, "repo");
        assert_eq!(dep.namespace, Some("user".to_string()));
    }

    #[test]
    fn test_dependency_extract_name_and_namespace() {
        // GitHub URL format
        let (name, namespace) =
            Dependency::extract_name_and_namespace("https://github.com/user/repo");
        assert_eq!(name, "repo");
        assert_eq!(namespace, Some("user".to_string()));

        // GitLab URL format
        let (name, namespace) =
            Dependency::extract_name_and_namespace("https://gitlab.com/group/project");
        assert_eq!(name, "project");
        assert_eq!(namespace, Some("group".to_string()));

        // URL with .git suffix
        let (name, namespace) =
            Dependency::extract_name_and_namespace("https://github.com/user/repo.git");
        assert_eq!(name, "repo");
        assert_eq!(namespace, Some("user".to_string()));

        // Invalid URL format - should handle gracefully
        let (name, namespace) = Dependency::extract_name_and_namespace("invalid_url");
        assert_eq!(name, "invalid_url");
        assert_eq!(namespace, None);
    }

    #[test]
    fn test_get_full_name() {
        let dep = Dependency::new(
            "https://github.com/user/repo".to_string(),
            "1.0.0".to_string(),
        );
        assert_eq!(dep.get_full_name(), "user/repo");

        // Test with no namespace
        let mut dep_no_namespace = dep;
        dep_no_namespace.namespace = None;
        assert_eq!(dep_no_namespace.get_full_name(), "repo");
    }

    #[test]
    fn test_dependencies_add_and_get() {
        let mut deps = Dependencies::new();
        let dep1 = Dependency::new(
            "https://github.com/user/repo1".to_string(),
            "1.0.0".to_string(),
        );
        let dep2 = Dependency::new(
            "https://github.com/user/repo2".to_string(),
            "2.0.0".to_string(),
        );

        // Initially empty
        assert_eq!(deps.len(), 0);
        assert!(deps.is_empty());

        // Add first dependency
        deps.add(dep1.clone());
        assert_eq!(deps.len(), 1);
        assert!(!deps.is_empty());

        // Check by name and namespace
        let found_dep = deps.get_by_name_and_namespace(&dep1.name, &dep1.namespace);
        assert!(found_dep.is_some());
        assert_eq!(found_dep.unwrap().url, "https://github.com/user/repo1");

        // Add second dependency
        deps.add(dep2);
        assert_eq!(deps.len(), 2);

        // Check get_all
        let all_deps = deps.get_all();
        assert_eq!(all_deps.len(), 2);
    }

    #[test]
    fn test_dependencies_update_and_remove() {
        let mut deps = Dependencies::new();
        let dep1 = Dependency::new(
            "https://github.com/user/repo1".to_string(),
            "1.0.0".to_string(),
        );

        // Add dependency
        deps.add(dep1);

        // Update by adding with same name/namespace but different version
        let updated_dep = Dependency::new(
            "https://github.com/user/repo1".to_string(),
            "2.0.0".to_string(),
        );
        deps.add(updated_dep);

        // Check that it was updated, not added as second
        assert_eq!(deps.len(), 1);
        let found_dep = deps.get_by_name_and_namespace("repo1", &Some("user".to_string()));
        assert!(found_dep.is_some());
        assert_eq!(found_dep.unwrap().version, "2.0.0");

        // Remove the dependency
        let removed = deps.remove("repo1", &Some("user".to_string()));
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().version, "2.0.0");
        assert_eq!(deps.len(), 0);
        assert!(deps.is_empty());
    }

    #[test]
    fn test_find_by_name_and_namespace() {
        let mut deps = Dependencies::new();

        // Add some dependencies with different names/namespaces
        deps.add(Dependency::new(
            "https://github.com/user1/repo1".to_string(),
            "1.0.0".to_string(),
        ));
        deps.add(Dependency::new(
            "https://github.com/user2/repo1".to_string(),
            "1.0.0".to_string(),
        ));
        deps.add(Dependency::new(
            "https://github.com/user1/repo2".to_string(),
            "1.0.0".to_string(),
        ));

        // Test finding by name and namespace
        assert!(
            deps.find_by_name_and_namespace("repo1", &Some("user1".to_string()))
                .is_some()
        );
        assert!(
            deps.find_by_name_and_namespace("repo1", &Some("user2".to_string()))
                .is_some()
        );
        assert!(
            deps.find_by_name_and_namespace("repo2", &Some("user1".to_string()))
                .is_some()
        );

        // Test non-existent combinations
        assert!(
            deps.find_by_name_and_namespace("repo2", &Some("user2".to_string()))
                .is_none()
        );
        assert!(
            deps.find_by_name_and_namespace("repo3", &Some("user1".to_string()))
                .is_none()
        );
    }
}
