use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::commons::{
    git::fetch_remote_git_repository_with_version,
    utilities::{construct_dependency_path, copy_dir_all, extract_name_and_namespace},
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
    pub namespace: String,
}

impl Dependency {
    /// Creates a new dependency with the specified repository URL and version
    pub fn new(url: String, version: String) -> Result<Self, Error> {
        let (name, namespace) = extract_name_and_namespace(&url)?;
        Ok(Self {
            url,
            version,
            name,
            namespace,
        })
    }

    /// Gets the fully qualified name including namespace if available
    pub fn get_full_name(&self) -> String {
        format!("{}/{}", self.namespace, self.name)
    }
    
    pub fn update(
        &mut self,
        package_path: &Path,
        version: Option<&str>,
    ) -> Result<(), Error> {
        let dependency_path: PathBuf = construct_dependency_path(package_path, &self.namespace, &self.name)?;

        // Remove existing directory to ensure a clean slate before (re)install.
        if dependency_path.exists() {
            std::fs::remove_dir_all(&dependency_path)?;
        }

        // Determine what version will be used (either forced or from package.json).
        let new_version: String = version.unwrap_or(&self.version).to_owned();
        let updated_dependency: Dependency = Dependency::new(self.url.clone(), new_version)?;

        // 4. Clone or copy the dependency from either a remote git repo or local path.
        let is_local: bool = Path::new(&updated_dependency.url).exists();
        
        if is_local {
            // Check that the local path is a valid library
            let local_pkg: Package = Package::from_file(Path::new(&updated_dependency.url))?;
            if !local_pkg.is_library {
                return Err(anyhow!(
                    "Package '{}' is not marked as a library. Only libraries can be added as dependencies.",
                    local_pkg.get_full_name()
                ));
            }
            copy_dir_all(Path::new(&updated_dependency.url), &dependency_path)?;
        }

        if !is_local {
            // Fetch from remote git repository
            fetch_remote_git_repository_with_version(
                &updated_dependency.url,
                Some(&updated_dependency.version),
                Some(&dependency_path),
            )?;
            // Validate that what we downloaded is a proper SPM library
            let remote_pkg: Package = Package::from_file(&dependency_path)?;
            if !remote_pkg.is_library {
                std::fs::remove_dir_all(&dependency_path)?;
                return Err(anyhow!(
                    "Package '{}' is not marked as a library. Only libraries can be added as dependencies.",
                    remote_pkg.get_full_name()
                ));
            }
        }
        
        // 5. Update the dependency version
        self.version = updated_dependency.version.clone();

        Ok(())
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
    pub fn remove(&mut self, name: &str, namespace: &str) -> Option<Dependency> {
        if let Some(index) = self.find_by_name_and_namespace(name, namespace) {
            Some(self.0.remove(index))
        } else {
            None
        }
    }

    /// Finds a dependency by name and namespace
    pub fn find_by_name_and_namespace(&self, name: &str, namespace: &str) -> Option<usize> {
        for (index, dependency) in self.0.iter().enumerate() {
            if dependency.name == name && namespace == &dependency.namespace {
                return Some(index);
            }
        }
        None
    }
    
    /// Finds a dependency by URL
    pub fn find_by_url(&self, url: &str) -> Option<usize> {
        for (index, dependency) in self.0.iter().enumerate() {
            if dependency.url == url {
                return Some(index);
            }
        }
        
        None
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
}
