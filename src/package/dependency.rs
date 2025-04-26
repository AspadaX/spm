use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::Path};

use crate::{
    commons::{
        git::fetch_remote_git_repository_with_version,
        utilities::{copy_dir_all, extract_name_and_namespace},
    },
    properties::{DEFAULT_DEPENDENCIES_FOLDER, DEFAULT_PACKAGE_JSON},
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
        namespace: &str,
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

        let dependency_dir_name: String = format!("{}/{}", dependency.namespace, dependency.name);
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
        let updated_dependency: Dependency = Dependency::new(dependency.url.clone(), new_version)?;

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
