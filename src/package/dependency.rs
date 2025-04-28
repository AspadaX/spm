use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::commons::{
    git::fetch_remote_git_repository_with_version,
    utilities::{construct_dependency_path, copy_dir_all, extract_name_and_namespace},
};

use super::Package;

/// Represents a single dependency with repository URL and version
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    pub fn update(&mut self, package_path: &Path, version: Option<&str>) -> Result<(), Error> {
        let dependency_path: PathBuf =
            construct_dependency_path(package_path, &self.namespace, &self.name)?;
        println!("{}", dependency_path.display());

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
pub struct Dependencies(HashSet<Dependency>);

impl Dependencies {
    /// Creates a new empty dependencies collection
    #[allow(dead_code)]
    pub fn new(dependencies: HashSet<Dependency>) -> Self {
        Self(dependencies)
    }

    /// Adds a dependency to the collection
    pub fn add(&mut self, dependency: Dependency) {
        // The HashSet will automatically handle duplicates with the same URL and version
        // since Dependency implements PartialEq and Eq

        // First, remove any existing dependency with the same name and namespace
        if let Some(index) =
            self.find_by_name_and_namespace(&dependency.name, &dependency.namespace)
        {
            // With HashSet we need to remove the old entry first
            let dep_to_remove = self.0.iter().nth(index).cloned();
            if let Some(dep) = dep_to_remove {
                self.0.remove(&dep);
            }
        }

        // Insert the new dependency
        self.0.insert(dependency);
    }
    
    /// Removes a dependency from the collection by name and namespace.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the dependency to remove.
    /// * `namespace` - The namespace of the dependency to remove.
    ///
    /// # Returns
    ///
    /// Returns `Some(Dependency)` if the dependency was found and removed, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use spm_rs::dependencies::{Dependencies, Dependency};
    ///
    /// let mut dependencies = Dependencies::new(HashSet::new());
    /// let dep1 = Dependency::new("https://github.com/test/package1".to_string(), "1.0.0".to_string()).unwrap();
    /// let dep2 = Dependency::new("https://github.com/test/package2".to_string(), "2.0.0".to_string()).unwrap();
    /// dependencies.add(dep1.clone());
    /// dependencies.add(dep2.clone());
    ///
    /// let removed_dependency = dependencies.remove("package1", "test");
    ///
    /// assert_eq!(removed_dependency, Some(dep1));
    /// assert_eq!(dependencies.len(), 1);
    /// ```
    pub fn remove(&mut self, name: &str, namespace: &str) -> Option<Dependency> {
        // Find the dependency with the given name and namespace
        let dep_to_remove = self
            .0
            .iter()
            .find(|dep| dep.name == name && dep.namespace == namespace)
            .cloned();

        // If found, remove it from the HashSet and return it
        if let Some(dep) = dep_to_remove {
            self.0.remove(&dep);
            Some(dep)
        } else {
            None
        }
    }

    /// Finds a dependency by name and namespace
    pub fn find_by_name_and_namespace(&self, name: &str, namespace: &str) -> Option<usize> {
        self.0
            .iter()
            .position(|dependency| dependency.name == name && namespace == &dependency.namespace)
    }

    /// Returns all dependencies
    pub fn get_all(&self) -> &HashSet<Dependency> {
        &self.0
    }

    /// Returns mutable references to all dependencies
    pub fn get_all_mut(&mut self) -> &mut HashSet<Dependency> {
        &mut self.0
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
}
