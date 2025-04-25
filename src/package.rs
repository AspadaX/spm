use std::io::Write;
use std::{
    fs::{DirEntry, File},
    path::{Path, PathBuf},
};

use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::properties::{DEFAULT_EXECUTABLE_ENTRYPOINT, DEFAULT_LIBRARUY_ENTRYPOINT, DEFAULT_PACKAGE_JSON, DEFAULT_SPM_FOLDER, DEFAULT_SPM_PACKAGES_FOLDER};
use crate::shell::ShellType;

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
        self.package_json_content
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

    pub fn get_main_entry_point(&self) -> &str {
        self.path_to_entrypoint.as_os_str().to_str().unwrap()
    }

    pub fn get_namespace(&self) -> Option<&String> {
        self.package_json_content.namespace.as_ref()
    }

    pub fn get_full_name(&self) -> String {
        self.package_json_content.get_full_name()
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
    // The namespace for the package (optional). In the format of `namespace`.
    namespace: Option<String>,
    // A brief description of the project
    description: String,
    // The project version, adhering to semantic versioning (semver)
    version: String,
    // The interpreter used for this project
    interpreter: ShellType,
    // Whether this is a library package
    #[serde(default)]
    is_library: bool,
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
            namespace: Some("default-namespace".to_string()),
            description: "Default description".to_string(),
            version: "0.1.0".to_string(),
            entrypoint: DEFAULT_EXECUTABLE_ENTRYPOINT.to_string(),
            interpreter: ShellType::Sh, // `spm` favors sh to be the default
            is_library: false,
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
    pub fn new(name: String, is_library: bool, interpreter: ShellType) -> Self {
        let entrypoint: String = if is_library {
            String::from(DEFAULT_LIBRARUY_ENTRYPOINT)
        } else {
            String::from(DEFAULT_EXECUTABLE_ENTRYPOINT)
        };

        Self {
            name,
            namespace: Some("default-namespace".to_string()), // Default namespace
            entrypoint,
            interpreter,
            is_library,
            ..Default::default()
        }
    }

    pub fn new_with_namespace(
        name: String,
        namespace: String,
        is_library: bool,
        interpreter: ShellType,
    ) -> Self {
        let entrypoint: String = if is_library {
            String::from(DEFAULT_LIBRARUY_ENTRYPOINT)
        } else {
            String::from(DEFAULT_EXECUTABLE_ENTRYPOINT)
        };

        Self {
            name,
            namespace: Some(namespace),
            entrypoint,
            interpreter,
            is_library,
            ..Default::default()
        }
    }

    /// Load `package.json`
    pub fn from_file(file: &Path) -> Result<Self, Error> {
        let package_json_path: PathBuf = file.join(DEFAULT_PACKAGE_JSON);
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

    pub fn get_full_name(&self) -> String {
        match &self.namespace {
            Some(namespace) => format!("{}/{}", namespace, self.name),
            None => self.name.clone(),
        }
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
            .join(DEFAULT_SPM_FOLDER);

        if !root_directory.exists() {
            // Temporarily use this way to create a `packages` folder. It will need to be
            // groupped to somewhere later.
            match std::fs::create_dir_all(&root_directory.join("packages")) {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!(
                        "Failed to create {} directory: {}",
                        DEFAULT_SPM_FOLDER,
                        e
                    ));
                }
            }
        }

        Ok(Self {
            root_directory,
            shell_type,
        })
    }

    /// Returns the path to the binary directory where executable scripts are symlinked.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `PathBuf` to the binary directory, or an `Error` if there was a problem.
    pub fn get_bin_directory(&self) -> Result<PathBuf, Error> {
        let bin_dir = self.root_directory.join("bin");

        // Create the bin directory if it doesn't exist
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir)?;
        }

        Ok(bin_dir)
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

        // Check if package_name contains a namespace (format: namespace/package)
        if package_name.contains('/') {
            let parts: Vec<&str> = package_name.split('/').collect();
            if parts.len() == 2 {
                let namespace = parts[0];
                let name = parts[1];

                // Try to find a package with matching namespace and name
                for package in installed_packages {
                    if let Some(pkg_namespace) = package.get_namespace() {
                        if pkg_namespace == namespace && package.get_pacakge_name() == name {
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
            if package.get_pacakge_name() == package_name {
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
    /// Create a package locally on the disk.
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

        // Get the shebang based on the interpreter set in `package.json`
        let shebang: &str = package.interpreter.get_shebang();

        // Create entrypoint script (either main.sh or lib.sh) based on whether it's a library
        let script_content: String;
        let script_filename: &str;

        if package.is_library {
            // Library script content
            script_filename = DEFAULT_LIBRARUY_ENTRYPOINT;
            script_content = format!(
                "{}\n\n# This is a library package\n# Define your functions below\n\ngreet() {{\n    echo \"Hello from the library!\"\n}}\n",
                shebang
            );
        } else {
            // Main script content
            script_filename = DEFAULT_EXECUTABLE_ENTRYPOINT;
            script_content = format!(
                "{}\n\nmain() {{\n    echo \"Hello World!\"\n}}\n\nmain",
                shebang
            );
        }

        // Create the entrypoint script
        match std::fs::File::create_new(path_to_package.join(script_filename)) {
            Ok(mut file) => {
                file.write_fmt(format_args!("{}", script_content))?;
            }
            Err(_) => {
                return Err(anyhow!(
                    "A `{}` file already exists in this directory. Please remove or rename it before proceeding!",
                    script_filename
                ));
            }
        };

        // Create a `package.json`
        match std::fs::File::create_new(path_to_package.join(DEFAULT_PACKAGE_JSON)) {
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
                file.write_all(
                    format!("{}\n\necho \"Setting up the package...\"", shebang).as_bytes(),
                )?;
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
                file.write_all(
                    format!("{}\n\necho \"Uninstalling the package...\"", shebang).as_bytes(),
                )?;
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
            PackageManager::copy_dir_all(path_to_package, &destination)?;
        }

        let setup_script_path: PathBuf = destination.join(package.install.setup_script);
        if setup_script_path.is_file() {
            std::process::Command::new(self.shell_type.to_string())
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

        if item.file_name().to_string_lossy().to_string() == DEFAULT_PACKAGE_JSON {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Normalize a package name. Use this when not sure about whether the package naming is
/// going to be absolutely correct.
pub fn normalize_package_name(name: &str) -> String {
    let standardized_separator: &str = "-";

    // Replace underscores with hyphens
    let mut normalized_name = name.replace("_", standardized_separator);

    // Replace uppercase letters with lowercase prefixed by a hyphen
    normalized_name = normalized_name
        .chars()
        .flat_map(|c| {
            if c.is_uppercase() {
                vec![
                    standardized_separator.to_string(),
                    c.to_lowercase().to_string(),
                ]
            } else {
                vec![c.to_string()]
            }
        })
        .collect::<String>();

    // Remove leading hyphen if present
    normalized_name
        .trim_start_matches(standardized_separator)
        .to_string()
}
