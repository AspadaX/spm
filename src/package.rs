use std::io::Write;
use std::{
    fs::{DirEntry, File}, path::{Path, PathBuf}
};

use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::shell::{ShellType, WhichInterpreter};

/// Represent the package's metadata
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct PackageMetadata {
    package_json_content: Package,
    path_to_package: PathBuf,
    path_to_entrypoint: PathBuf,
    path_to_setup_script: PathBuf,
    path_to_uninstall_script: PathBuf,
}

impl Into<Package> for PackageMetadata {
    fn into(self) -> Package {
        Package {
            name: self.package_json_content.name,
            description: self.package_json_content.description,
            version: self.package_json_content.version,
            entrypoint: self.package_json_content.entrypoint,
            install: self.package_json_content.install.clone(),
            uninstall: self.package_json_content.uninstall.clone(),
        }
    }
}

impl PackageMetadata {
    pub fn get_pacakge_name(&self) -> &str {
        &self.package_json_content.name
    }
    
    pub fn get_description(&self) -> &str {
        &self.package_json_content.description
    }

    pub fn get_version(&self) -> &str {
        &self.package_json_content.version
    }
}

/// Represent the installation options
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstallationOptions {
    // Script to run when using `spm run`
    setup_script: String,
    // Whether to register this in the environment variables,
    // default to false
    register_to_environment_tool: bool,
}

/// Represent the `package.json` file under each shell script project's
/// root directory.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Package {
    // The name of the project. In the format of `package-name`.
    name: String,
    // A brief description of the project
    description: String,
    // The project version, adhering to semantic versioning (semver)
    version: String,
    // The shell script executed with `spm run`
    entrypoint: String,
    // Configuration for actions during package installation
    install: InstallationOptions,
    // The script executed during package uninstallation
    uninstall: String,
}

impl Default for Package {
    fn default() -> Self {
        Package {
            name: "".to_string(),
            description: "Default description".to_string(),
            version: "0.1.0".to_string(),
            entrypoint: "main.sh".to_string(),
            install: InstallationOptions {
                setup_script: "install.sh".to_string(),
                register_to_environment_tool: false,
            },
            uninstall: "uninstall.sh".to_string(),
        }
    }
}

impl From<File> for Package {
    fn from(value: File) -> Self {
        let package: Package =
            serde_json::from_reader(value).expect("Failed to parse JSON file into Package");
        package
    }
}

impl Package {
    pub fn new(name: String, is_library: bool) -> Self {
        let entrypoint: String = if is_library {
            String::from("lib.sh")
        } else {
            String::from("main.sh")
        };

        Self {
            name,
            entrypoint,
            ..Default::default()
        }
    }

    /// Load `package.json`
    pub fn from_file(file: &Path) -> Result<Self, Error> {
        let package_json_path: PathBuf = file.join("package.json");
        if !package_json_path.is_file() {
            return Err(anyhow!(
                "The package.json file is missing in the provided package path"
            ));
        }

        Ok(serde_json::from_reader(File::open(&package_json_path)?)?)
    }

    pub fn access_main_entrypoint(&self) -> &str {
        &self.entrypoint
    }
}

#[derive(Debug, Clone)]
pub struct PackageManager {
    shell_type: ShellType,
    root_directory: PathBuf,
}

impl PackageManager {
    pub fn new() -> Result<Self, Error> {
        let shell_type = if cfg!(target_os = "windows") {
            ShellType::Cmd
        } else {
            ShellType::Bash
        };
        
        let root_directory: PathBuf = dirs::home_dir()
            .ok_or_else(|| anyhow!("Failed to locate home directory"))?
            .join(".spm");
        
        if !root_directory.exists() {
            // Temporarily use this way to create a `packages` folder. It will need to be 
            // groupped to somewhere later. 
            match std::fs::create_dir_all(&root_directory.join("packages")) {
                Ok(_) => (),
                Err(e) => return Err(anyhow!("Failed to create .spm directory: {}", e)),
            }
        }

        Ok(Self {
            root_directory,
            shell_type,
        })
    }
    
    /// Retrieves a `PackageMetadata` object by its name.
    ///
    /// This function searches through the installed packages and returns the `PackageMetadata`
    /// of the package that matches the provided name.
    ///
    /// # Arguments
    ///
    /// * `package_name` - A `String` representing the name of the package to search for.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok(PackageMetadata)` if the package is found.
    /// - `Err(Error)` if the package is not found or if an error occurs during the search.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The package installation directory cannot be accessed.
    /// - The package with the specified name is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use your_crate_name::{PackageManager, PackageMetadata};
    ///
    /// let package_manager = PackageManager::new().unwrap();
    /// let package_name = "example_package".to_string();
    ///
    /// match package_manager.get_package_by_name(package_name) {
    ///     Ok(package_metadata) => {
    ///         println!("Found package: {}", package_metadata.get_pacakge_name());
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn get_package_by_name(&self, package_name: String) -> Result<PackageMetadata, Error> {
        let installed_packages: Vec<PackageMetadata> = self.get_installed_packages()?;
        for package in installed_packages {
            if package.get_pacakge_name() == package_name {
                return Ok(package);
            }
        }
        Err(anyhow!("Package with name '{}' not found", package_name))
    }

    pub fn keyword_search(&self, keywords: String) -> Result<Vec<PackageMetadata>, Error> {
        let words: Vec<String> = keywords
            .split(" ")
            .map(|keyword: &str| keyword.to_lowercase())
            .collect();
        let mut matched_packages: Vec<(PackageMetadata, usize)> = Vec::new();

        if let Ok(packages) = self.get_installed_packages() {
            for package in packages {
                let package_words: Vec<String> = package.package_json_content.name
                    .split("-")
                    .map(|item| item.to_string())
                    .collect();
                
                if package_words.is_empty() {
                    continue;
                }
                
                for word in words.iter() {
                    // Skip if the keyword is empty
                    if word.is_empty() {
                        continue;
                    }
                    
                    // When a keyword is found in the name
                    if package_words.contains(word) {
                        let mut is_existing: bool = false;
                        // Increment the match count if the package is already in the list
                        for matched_package in &mut matched_packages {
                            if matched_package.0 == package {
                                matched_package.1 += 1;
                                is_existing = true;
                            }
                        }
                        
                        // Add the package to the list if the package is not already in the list
                        if !is_existing {
                            matched_packages.push(
                                (package.clone(), 1)
                            );
                        }
                        
                        continue;
                    }
                }
            }
        }

        // Sort the packages by match count in descending order
        matched_packages
            .sort_by(
                |a, b| 
                b.1.cmp(&a.1)
            );
        
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
    ///
    /// This directory is located under the root directory of the package manager
    /// (typically `~/.spm/packages`).
    ///
    /// # Returns
    /// A `PathBuf` representing the full path to the package installation directory.
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use your_crate_name::PackageManager;
    ///
    /// let package_manager = PackageManager::new().unwrap();
    /// let installation_dir = package_manager.access_package_installation_directory();
    /// assert_eq!(installation_dir, PathBuf::from("~/.spm/packages").expand());
    /// ```
    pub fn access_package_installation_directory(&self) -> PathBuf {
        self.root_directory.join("packages")
    }

    /// Returns the full path to the entrypoint script of the package.
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use your_crate_name::Package;
    ///
    /// let package = Package::new("example_package".to_string(), false);
    /// let path = PathBuf::from("/path/to/package").join(&package.entrypoint);
    /// assert_eq!(path.to_str().unwrap(), "/path/to/package/main.sh");
    /// ```
    pub fn create_package(&self, path_to_package: &Path, package: &Package) -> Result<(), Error> {
        if !path_to_package.is_dir() {
            return Err(anyhow!(
                "A shell script project must be initialized in a directory!"
            ));
        }

        // Create a `src` folder
        std::fs::create_dir(path_to_package.join("src"))?;

        // Create a `main.sh` with shebang and hello world in it
        let main_script_content =
            String::from("#! /bin/bash\n\nmain() {\n    echo \"Hello World!\"\n}\n\nmain");
        match std::fs::File::create_new(path_to_package.join("main.sh")) {
            Ok(mut file) => {
                file.write_fmt(format_args!("{}", main_script_content))?;
            }
            Err(_) => {
                return Err(anyhow!(
                    "A `main.sh` file already exists in this directory. Please remove or rename it before proceeding!"
                ));
            }
        };

        // Create a `package.json`
        match std::fs::File::create_new(path_to_package.join("package.json")) {
            Ok(file) => {
                serde_json::to_writer_pretty(file, package)?;
            }
            Err(_) => {
                return Err(anyhow!(
                    "A `package.json` file already exists in this directory. Please remove or rename it before proceeding!"
                ));
            }
        }

        // Create a setup script
        let setup_script_content: &String = &package.install.setup_script;
        match std::fs::File::create_new(path_to_package.join(setup_script_content)) {
            Ok(mut file) => {
                file.write_all(b"#!/bin/bash\n\necho \"Setting up the package...\"")?;
            }
            Err(_) => {
                return Err(anyhow!(
                    "A setup script file already exists in this directory. Please remove or rename it before proceeding!"
                ));
            }
        };

        // Create an uninstall script
        let uninstall_script_content: &String = &package.uninstall;
        match std::fs::File::create_new(path_to_package.join(uninstall_script_content)) {
            Ok(mut file) => {
                file.write_all(b"#!/bin/bash\n\necho \"Uninstalling the package...\"")?;
            }
            Err(_) => {
                return Err(anyhow!(
                    "An uninstall script file already exists in this directory. Please remove or rename it before proceeding!"
                ));
            }
        };

        Ok(())
    }

    /// Retrieves the list of installed packages by scanning the package installation directory (`~/.spm/packages`).
    ///
    /// # Returns
    /// A `Result` containing a vector of `PackageMetadata` for all installed packages, or an `Error` if the operation fails.
    ///
    /// # Examples
    /// ```
    /// use your_crate_name::Package;
    /// use std::path::PathBuf;
    ///
    /// let installed_packages = Package::get_installed_packages();
    /// match installed_packages {
    ///     Ok(packages) => {
    ///         for package in packages {
    ///             println!("Installed package: {}", package.package_json_content.name);
    ///         }
    ///     }
    ///     Err(e) => eprintln!("Error retrieving installed packages: {}", e),
    /// }
    /// ```
    pub fn get_installed_packages(&self) -> Result<Vec<PackageMetadata>, Error> {
        let spm_dir: PathBuf = self.access_package_installation_directory();

        if !spm_dir.is_dir() {
            return Err(anyhow!(
                "The package installation directory `~/.spm/packages` does not exist"
            ));
        }

        let mut installed_packages: Vec<PackageMetadata> = Vec::new();

        for entry in std::fs::read_dir(spm_dir)? {
            let entry: DirEntry = entry?;
            let path: PathBuf = entry.path();

            if path.is_dir() {
                let package_json_path: PathBuf = path.join("package.json");

                if package_json_path.is_file() {
                    let package: Package =
                        serde_json::from_reader(File::open(&package_json_path)?)?;
                    installed_packages.push(PackageMetadata {
                        package_json_content: package.clone(),
                        path_to_package: path.clone(),
                        path_to_entrypoint: path.join(&package.entrypoint),
                        path_to_setup_script: path.join(&package.install.setup_script),
                        path_to_uninstall_script: path.join(&package.uninstall),
                    });
                }
            }
        }

        Ok(installed_packages)
    }

    /// Installs a package by copying or moving it to the package installation directory
    /// and executing its setup script if available.
    ///
    /// # Arguments
    ///
    /// * `path_to_package` - A reference to the path of the package to be installed.
    /// * `is_move` - A boolean indicating whether to move (`true`) or copy (`false`) the package.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. Returns `Ok(())` on success, or an `Error` if
    /// any part of the operation fails.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The package path is invalid.
    /// - The package installation directory cannot be created.
    /// - The package cannot be moved or copied to the destination.
    /// - The setup script cannot be executed or is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use your_crate_name::PackageManager;
    /// use your_crate_name::ShellType;
    ///
    /// let package_manager = PackageManager::new(ShellType::Bash).unwrap();
    /// let path_to_package = Path::new("/path/to/package");
    ///
    /// match package_manager.install_package(path_to_package, false) {
    ///     Ok(_) => println!("Package installed successfully!"),
    ///     Err(e) => eprintln!("Failed to install package: {}", e),
    /// }
    /// ```
    pub fn install_package(&self, path_to_package: &Path, is_move: bool) -> Result<(), Error> {
        let spm_dir: PathBuf = self.access_package_installation_directory();
        let package = Package::from_file(path_to_package)?;

        if !spm_dir.exists() {
            std::fs::create_dir_all(&spm_dir)?;
        }

        let destination: PathBuf = spm_dir.join(
            path_to_package
                .file_name()
                .ok_or_else(|| anyhow!("Invalid package path"))?,
        );

        if is_move {
            std::fs::rename(path_to_package, &destination)?;
        } else {
            PackageManager::copy_dir_all(path_to_package, &destination)?;
        }

        let setup_script_path: PathBuf = destination.join(package.install.setup_script);
        if setup_script_path.is_file() {
            std::process::Command::new(self.shell_type.get_intepreter())
                .arg(setup_script_path)
                .status()
                .map_err(|e| anyhow!("Failed to execute setup script: {}", e))?;
        } else {
            return Err(anyhow!("Setup script not found in the package"));
        }

        Ok(())
    }

    /// Recursively copies the contents of a directory from the source path to the destination path.
    ///
    /// This function ensures that all files and subdirectories in the source directory are copied
    /// to the destination directory, preserving the directory structure.
    ///
    /// # Arguments
    ///
    /// * `src` - A reference to the source directory path.
    /// * `dst` - A reference to the destination directory path.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. Returns `Ok(())` on success, or an `Error` if
    /// any part of the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use your_crate_name::PackageManager;
    ///
    /// let src = Path::new("/path/to/source");
    /// let dst = Path::new("/path/to/destination");
    ///
    /// match PackageManager::copy_dir_all(src, dst) {
    ///     Ok(_) => println!("Directory copied successfully!"),
    ///     Err(e) => eprintln!("Failed to copy directory: {}", e),
    /// }
    /// ```
    fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), Error> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry: DirEntry = entry?;
            let entry_path: PathBuf = entry.path();
            let dest_path: PathBuf = dst.join(entry.file_name());
            if entry_path.is_dir() {
                PackageManager::copy_dir_all(&entry_path, &dest_path)?;
            } else {
                std::fs::copy(&entry_path, &dest_path)?;
            }
        }
        Ok(())
    }

    /// Uninstalls a package by removing its directory and executing its uninstall script if available.
    ///
    /// # Arguments
    ///
    /// * `path_to_package` - A reference to the path of the package to be uninstalled.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. Returns `Ok(())` on success, or an `Error` if
    /// any part of the operation fails.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The package path is invalid or does not exist.
    /// - The uninstall script cannot be executed or is missing.
    /// - The package directory cannot be removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use your_crate_name::PackageManager;
    ///
    /// let path_to_package = Path::new("/path/to/package");
    ///
    /// match PackageManager::uninstall_package(path_to_package) {
    ///     Ok(_) => println!("Package uninstalled successfully!"),
    ///     Err(e) => eprintln!("Failed to uninstall package: {}", e),
    /// }
    /// ```
    fn uninstall_package(&self, path_to_package: &Path) -> Result<(), Error> {
        let package = Package::from_file(path_to_package)?;

        if !path_to_package.exists() {
            return Err(anyhow!("The specified package path does not exist"));
        }

        let uninstall_script_path: PathBuf = path_to_package.join(package.uninstall);
        if uninstall_script_path.is_file() {
            std::process::Command::new("sh")
                .arg(uninstall_script_path)
                .status()
                .map_err(|e| anyhow!("Failed to execute uninstall script: {}", e))?;
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

/// Checks if a given directory contains a `package.json` file, indicating it is a package.
///
/// # Arguments
///
/// * `path` - A reference to the path of the directory to check.
///
/// # Returns
///
/// A `Result` containing a boolean value:
/// - `true` if the directory contains a `package.json` file.
/// - `false` otherwise.
///
/// # Errors
///
/// Returns an `Error` if the directory cannot be read or if any IO operation fails.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use your_crate_name::is_inside_a_package;
///
/// let path = Path::new("/path/to/directory");
/// match is_inside_a_package(path) {
///     Ok(is_package) => {
///         if is_package {
///             println!("The directory is a package.");
///         } else {
///             println!("The directory is not a package.");
///         }
///     }
///     Err(e) => eprintln!("Error checking directory: {}", e),
/// }
/// ```
pub fn is_inside_a_package(path: &Path) -> Result<bool, Error> {
    let directory_items: std::fs::ReadDir = path.read_dir().unwrap();

    for item in directory_items {
        let item: DirEntry = item?;

        if item.file_name().to_string_lossy().to_string() == "package.json" {
            return Ok(true);
        }
    }

    Ok(false)
}
