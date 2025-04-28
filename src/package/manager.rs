use anyhow::{Error, Result, anyhow};
use std::fs::{DirEntry, File};
use std::path::{Path, PathBuf};

use crate::commons::git::fetch_remote_git_repository_with_version;
use crate::commons::utilities::{construct_dependency_path, copy_dir_all, extract_name_and_namespace};
use crate::properties::{
    DEFAULT_BIN_FOLDER, DEFAULT_DEPENDENCIES_FOLDER, DEFAULT_PACKAGE_JSON, DEFAULT_SPM_FOLDER, DEFAULT_SPM_PACKAGES_FOLDER
};
use crate::shell::{ExecutionContext, execute_shell_script_with_context};

use super::creator::create_package_structure;
use super::dependency::Dependency;
use super::metadata::{Package, PackageMetadata, normalize_package_name};

#[derive(Debug, Clone)]
pub struct PackageManager {
    root_directory: PathBuf,
}

impl PackageManager {
    pub fn new() -> Result<Self, Error> {
        let root_directory: PathBuf = dirs::home_dir()
            .ok_or_else(|| anyhow!("Failed to locate home directory"))?
            .join(DEFAULT_SPM_FOLDER);

        if !root_directory.exists() {
            // Create both packages and bin folders
            match std::fs::create_dir_all(&root_directory.join(DEFAULT_SPM_PACKAGES_FOLDER)) {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!(
                        "Failed to create {} directory: {}",
                        DEFAULT_SPM_FOLDER,
                        e
                    ));
                }
            }

            match std::fs::create_dir_all(&root_directory.join(DEFAULT_BIN_FOLDER)) {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!("Failed to create bin directory: {}", e));
                }
            }
        }

        Ok(Self { root_directory })
    }

    /// Returns the path to the binary directory where executable scripts are symlinked.
    pub fn get_bin_directory(&self) -> Result<PathBuf, Error> {
        let bin_dir: PathBuf = self.root_directory.join(DEFAULT_BIN_FOLDER);

        // Create the bin directory if it doesn't exist
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir)?;
        }

        Ok(bin_dir)
    }

    /// Retrieves a `PackageMetadata` object by its name.
    pub fn get_package_by_name(&self, package_name: String) -> Result<PackageMetadata, Error> {
        let installed_packages: Vec<PackageMetadata> = self.get_installed_packages()?;

        // Check if package_name contains a namespace (format: namespace/package)
        if package_name.contains('/') {
            let parts: Vec<&str> = package_name.split('/').collect();
            if parts.len() == 2 {
                let namespace = parts[0];
                let name = parts[1];

                // Try to find a package with matching namespace and name
                for package in installed_packages {
                    if let Some(pkg_namespace) = package.get_namespace() {
                        if pkg_namespace == namespace && package.get_package_name() == name {
                            return Ok(package);
                        }
                    }
                }
            }
            return Err(anyhow!("Package with name '{}' not found", package_name));
        }

        // If no namespace specified, first look for exact package name match in any namespace
        let mut matching_packages = Vec::new();

        for package in installed_packages {
            if package.get_package_name() == package_name {
                matching_packages.push(package);
            }
        }

        // If there's only one match, return it
        if matching_packages.len() == 1 {
            return Ok(matching_packages.remove(0));
        }
        // If there are multiple matches in different namespaces, return the first one
        else if matching_packages.len() > 1 {
            return Ok(matching_packages.remove(0));
        }

        // If no exact match found, return an error
        Err(anyhow!("Package with name '{}' not found", package_name))
    }

    pub fn keyword_search(&self, keywords: &str) -> Result<Vec<PackageMetadata>, Error> {
        let words: Vec<String> = keywords
            .split(",")
            .map(|keyword: &str| keyword.to_lowercase())
            .collect();
        let mut matched_packages: Vec<(PackageMetadata, usize)> = Vec::new();

        if let Ok(packages) = self.get_installed_packages() {
            for package in packages {
                // Create a full package name including namespace if it exists
                let full_name = package.get_full_name();
                let package_name: String =
                    normalize_package_name(&package.package_json_content.name);

                // Also normalize and search in namespace if one exists
                let mut namespace_words: Vec<String> = Vec::new();
                if let Some(namespace) = &package.package_json_content.namespace {
                    namespace_words = normalize_package_name(namespace)
                        .split("-")
                        .map(|item| item.to_string())
                        .collect();
                }

                // Extract words from package name for matching
                let package_words: Vec<String> = package_name
                    .split("-")
                    .map(|item| item.to_string())
                    .collect();

                if package_words.is_empty() && namespace_words.is_empty() {
                    continue;
                }

                // If exactly matches the input and the package name or full name
                if &package.package_json_content.name == keywords || &full_name == keywords {
                    matched_packages.push((package.clone(), 2)); // Higher score for exact match
                    continue;
                }

                if package_name == keywords {
                    matched_packages.push((package.clone(), 1));
                    continue;
                }

                let mut match_score = 0;

                for word in words.iter() {
                    // Skip if the keyword is empty
                    if word.is_empty() {
                        continue;
                    }

                    // When a keyword is found in the name
                    if package_words.contains(word) {
                        match_score += 1;
                    }

                    // When a keyword is found in the namespace
                    if namespace_words.contains(word) {
                        match_score += 1;
                    }
                }

                // Add the package with its match score if any matches found
                if match_score > 0 {
                    matched_packages.push((package.clone(), match_score));
                }
            }
        }

        // Sort the packages by match count in descending order
        matched_packages.sort_by(|a, b| b.1.cmp(&a.1));

        let mut results: Vec<PackageMetadata> = Vec::new();
        for matched_package in matched_packages {
            // Skip the packages if the score is zero
            if matched_package.1 != 0 {
                results.push(matched_package.0);
            }
        }

        Ok(results)
    }

    /// Returns the path to the package installation directory.
    pub fn access_package_installation_directory(&self) -> PathBuf {
        self.root_directory.join("packages")
    }

    /// Creates a package at the specified path with the provided package configuration
    pub fn create_package(&self, path_to_package: &Path, package: &Package) -> Result<(), Error> {
        create_package_structure(path_to_package, package)
    }

    /// Retrieves the list of installed packages
    pub fn get_installed_packages(&self) -> Result<Vec<PackageMetadata>, Error> {
        let spm_dir: PathBuf = self.access_package_installation_directory();

        if !spm_dir.is_dir() {
            return Err(anyhow!(format!(
                "The package installation directory `~/{}/{}` does not exist",
                DEFAULT_SPM_FOLDER, DEFAULT_SPM_PACKAGES_FOLDER
            )));
        }

        let mut installed_packages: Vec<PackageMetadata> = Vec::new();

        // Function to process package directories
        let process_package_dir = |path: PathBuf| -> Result<Option<PackageMetadata>, Error> {
            let package_json_path: PathBuf = path.join(DEFAULT_PACKAGE_JSON);
            if package_json_path.is_file() {
                let package: Package = serde_json::from_reader(File::open(&package_json_path)?)?;
                return Ok(Some(PackageMetadata {
                    package_json_content: package.clone(),
                    path_to_package: path.clone(),
                    path_to_entrypoint: path.join(&package.entrypoint),
                    path_to_setup_script: path.join(&package.install.setup_script),
                    path_to_uninstall_script: path.join(&package.uninstall),
                }));
            }
            Ok(None)
        };

        // Read the root packages directory
        for entry in std::fs::read_dir(spm_dir)? {
            let entry: DirEntry = entry?;
            let path: PathBuf = entry.path();

            if path.is_dir() {
                // Check if this is a namespace directory or a package directory
                let package_json_path: PathBuf = path.join(DEFAULT_PACKAGE_JSON);

                if package_json_path.is_file() {
                    // This is a package directory (non-namespaced package)
                    if let Some(metadata) = process_package_dir(path)? {
                        installed_packages.push(metadata);
                    }
                } else {
                    // This could be a namespace directory, check its subdirectories for packages
                    for namespace_entry in std::fs::read_dir(path)? {
                        let namespace_entry: DirEntry = namespace_entry?;
                        let namespace_path: PathBuf = namespace_entry.path();

                        if namespace_path.is_dir() {
                            if let Some(metadata) = process_package_dir(namespace_path)? {
                                installed_packages.push(metadata);
                            }
                        }
                    }
                }
            }
        }

        Ok(installed_packages)
    }

    /// Installs a package globally
    pub fn install_package(
        &self,
        path_to_package: &Path,
        is_move: bool,
        is_force: bool,
    ) -> Result<(), Error> {
        let spm_dir: PathBuf = self.access_package_installation_directory();
        let package: Package = Package::from_file(path_to_package)?;

        if !spm_dir.exists() {
            std::fs::create_dir_all(&spm_dir)?;
        }

        // Determine the destination path based on namespace
        let mut destination: PathBuf = spm_dir;
        if let Some(namespace) = &package.namespace {
            // Create namespace directory if it doesn't exist
            destination = destination.join(namespace);
            if !destination.exists() {
                std::fs::create_dir_all(&destination)?;
            }
        }

        // Add the package directory to the destination
        destination = destination.join(
            path_to_package
                .file_name()
                .ok_or_else(|| anyhow!("Invalid package path"))?,
        );

        // Check if this package is forced to overwrite the installed one
        if destination.exists() && !is_force {
            return Err(anyhow!(
                "The package already installed. Use `--force` (-F) flag to force an install or update"
            ));
        }

        // Whether to move the package to the installation directory.
        // Usually we move the package when it is cloned remotely from a git repository.
        if is_move {
            std::fs::rename(path_to_package, &destination)?;
        } else {
            copy_dir_all(path_to_package, &destination)?;
        }

        let setup_script_path: PathBuf = destination.join(package.install.setup_script);
        if setup_script_path.is_file() {
            // Execute the setup script
            execute_shell_script_with_context(
                setup_script_path.to_str().unwrap(),
                &[],
                ExecutionContext::ScriptDirectory,
            )?;
        } else {
            return Err(anyhow!("Setup script not found in the package"));
        }

        Ok(())
    }

    /// Uninstalls a package
    fn uninstall_package(&self, path_to_package: &Path) -> Result<(), Error> {
        let package = Package::from_file(path_to_package)?;

        if !path_to_package.exists() {
            return Err(anyhow!("The specified package path does not exist"));
        }

        let uninstall_script_path: PathBuf = path_to_package.join(package.uninstall);
        if uninstall_script_path.is_file() {
            // Execute the uninstall script
            execute_shell_script_with_context(
                uninstall_script_path.to_str().unwrap(),
                &[],
                ExecutionContext::ScriptDirectory,
            )?;
        } else {
            return Err(anyhow!("Uninstall script not found in the package"));
        }

        std::fs::remove_dir_all(path_to_package)
            .map_err(|e| anyhow!("Failed to remove package directory: {}", e))?;

        Ok(())
    }

    pub fn uninstall_package_by_name(&self, package_name: String) -> Result<(), Error> {
        let package_metadata: PackageMetadata = self.get_package_by_name(package_name)?;
        self.uninstall_package(package_metadata.path_to_package.as_path())
    }
}

/// When launching `spm` under a shell script project, 
/// this will be hanlding the package wide functionalities. 
#[derive(Debug, Clone)]
pub struct LocalPackageManager {
    package_json_path: PathBuf,
    root_directory: PathBuf,
    package: Package
}

impl LocalPackageManager {
    pub fn new(package_root_directory: PathBuf) -> Self {
        LocalPackageManager {
            package: Package::from_file(
                Path::new(
                    &std::env::current_dir().unwrap()
                )
            )
                .expect("Failed to load package.json from current directory"),
            package_json_path: package_root_directory.join(DEFAULT_PACKAGE_JSON),
            root_directory: package_root_directory,
        }
    }
    
    /// Adds a dependency to a local package
    pub fn add_dependency(
        &self,
        package_path: &Path,
        dependency_package_path: &Path,
        url: &str,
        version: &str,
    ) -> Result<(), Error> {
        // 1. Construct the Dependency object
        let dependency: Dependency = Dependency::new(url.to_string(), version.to_string())?;
        if dependency.name.is_empty() {
            return Err(anyhow!("Failed to extract valid repository name from URL"));
        }

        // 2. Create the dependencies folder if necessary
        let dependencies_dir: PathBuf = package_path.join(DEFAULT_DEPENDENCIES_FOLDER);
        if !dependencies_dir.exists() {
            return Err(anyhow!(
                "`{}` does not exist in the project root. Please ensure the project integrity",
                DEFAULT_DEPENDENCIES_FOLDER
            ));
        }

        // 3. Build the local installation path (namespace + name)
        let target_path: PathBuf = dependencies_dir
            .join(&dependency.namespace)
            .join(&dependency.name);

        // 4. Abort if a folder with the same name already exists
        if target_path.exists() {
            return Err(anyhow!(
                "Dependency '{}' is already installed. Consider updating instead.",
                dependency.get_full_name()
            ));
        }

        // Local folder → copy
        let pkg: Package = Package::from_file(dependency_package_path)?;
        if !pkg.is_library {
            return Err(anyhow!(
                "The package '{}' is not marked as a library. Only libraries can be added as dependencies.",
                pkg.get_full_name()
            ));
        }

        // Use the actual path of where the dependency package is located at.
        // We are currently downloading the package to a local place if it is a remote repo.
        copy_dir_all(Path::new(dependency_package_path), &target_path)?;

        // 6. Add the dependency to the current package’s package.json
        let mut package: Package = Package::from_file(package_path)?;
        package.dependencies.add(dependency);
        
        self.update_package_json();

        Ok(())
    }

    /// Removes a dependency from a local package
    pub fn remove_dependency(
        &self,
        package_path: &Path,
        name: &str,
        namespace: &str,
    ) -> Result<(), Error> {
        // Load the package.json
        let package_json_path: PathBuf = package_path.join(DEFAULT_PACKAGE_JSON);
        let mut package: Package = super::metadata::Package::from_file(package_path)?;
        // Reconstruct the correct name and namespace
        for dependency in package.access_dependencies().get_all_mut().iter_mut() {
            let (name, namespace) = extract_name_and_namespace(&dependency.url)?;
            dependency.name = name;
            dependency.namespace = namespace;
        }

        // Check if the dependency exists
        if package
            .dependencies
            .find_by_name_and_namespace(name, namespace)
            .is_none()
        {
            return Err(anyhow!(
                "Dependency '{}' with namespace '{}' not found in {}",
                name,
                namespace,
                DEFAULT_PACKAGE_JSON
            ));
        }

        // Remove the dependency from the package.json
        if let Some(removed_dep) = package.dependencies.remove(name, &namespace) {
            // Save the updated package.json
            let file: File = std::fs::File::create(&package_json_path)?;
            serde_json::to_writer_pretty(file, &package)?;

            // Construct dependency directory name
            let dependency_dir_name: String =
                format!("{}/{}", removed_dep.namespace, removed_dep.name);

            // Remove the dependency directory
            let dependency_path = package_path
                .join(DEFAULT_DEPENDENCIES_FOLDER)
                .join(&dependency_dir_name);
            if dependency_path.exists() {
                std::fs::remove_dir_all(&dependency_path)?;
            }

            return Ok(());
        }

        Err(anyhow!("Failed to remove dependency"))
    }

    /// Refreshes all dependencies or a specific dependency in a package
    /// If a dependency is manually added to package.json, this will install it
    /// Refreshes all dependencies or a specific dependency in a package
    /// If a dependency is manually added to package.json, this will install it
    pub fn refresh_dependencies(
        &mut self,
        package_path: &Path,
        version: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        let mut package: Package = Package::from_file(package_path)?;
        
        // This will hold the list of successfully refreshed dependency names
        let mut processed_dependencies: Vec<String> = Vec::new();
        
        // 3. For each dependency, remove any old copy, then clone/copy it again.
        for dependency in package.dependencies.get_all().iter() {
            dependency.update(package_path, version)?;
            processed_dependencies.push(dependency.get_full_name());
        }
        
        self.update_package_json()?;

        // 7. Return the complete set of processed dependency names
        Ok(processed_dependencies)
    }
    
    pub fn load_package_json(&mut self) -> Result<Package, Error> {
        if !self.package_json_path.exists() {
            return Err(anyhow!("Package.json file not found"));
        }
        let package: Package = Package::from_file(&self.package_json_path)?;
        Ok(package)
    }
    
    pub fn update_package_json(&self) -> Result<(), Error> {
        let file: File = File::create(&self.package_json_path)?;
        let package: Package = Package::from_file(&self.package_json_path)?;
        serde_json::to_writer_pretty(file, &package)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_manager_new() {
        let result = PackageManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_bin_directory() {
        let pm = PackageManager::new().unwrap();
        let result = pm.get_bin_directory();
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("bin"));
    }
}
