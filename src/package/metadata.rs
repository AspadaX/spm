use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{DirEntry, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use super::dependency::{Dependencies, Dependency};
use crate::properties::{
    DEFAULT_DEPENDENCIES_FOLDER, DEFAULT_EXECUTABLE_ENTRYPOINT, DEFAULT_LIBRARY_ENTRYPOINT, DEFAULT_PACKAGE_JSON, DEFAULT_SRC_FOLDER
};
use crate::shell::ShellType;

/// Represent the package's metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageMetadata {
    pub package_json_content: Package,
    pub path_to_package: PathBuf,
    pub path_to_entrypoint: PathBuf,
    pub path_to_setup_script: PathBuf,
    pub path_to_uninstall_script: PathBuf,
}

impl Into<Package> for PackageMetadata {
    fn into(self) -> Package {
        self.package_json_content
    }
}

impl PackageMetadata {
    pub fn get_package_name(&self) -> &str {
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

impl Ord for PackageMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by full name as the primary key
        let self_name = self.get_full_name();
        let other_name = other.get_full_name();
        self_name.cmp(&other_name)
    }
}

impl PartialOrd for PackageMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represent the installation options
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct InstallationOptions {
    // Script to run when using `spm run`
    pub setup_script: String,
    // Whether to register this in the environment variables,
    // default to false
    pub register_to_environment_tool: bool,
}

/// Represent the `package.json` file under each shell script project's
/// root directory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub description: String,
    pub version: String,
    pub namespace: Option<String>,
    pub interpreter: ShellType,
    pub entrypoint: String,
    pub install: InstallationOptions,
    pub uninstall: String,
    #[serde(default)]
    pub is_library: bool,
    #[serde(default)]
    pub dependencies: Dependencies,
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
            dependencies: Dependencies::default(),
        }
    }
}

impl From<File> for Package {
    fn from(value: File) -> Self {
        // Display the value content
        let mut contents: String = String::new();
        let mut file: File = value.try_clone().unwrap();
        file.read_to_string(&mut contents).expect("Failed to read package.json to string");
        let mut package: Package =
            serde_json::from_str(&contents)
                .expect("Failed to parse JSON file into Package");
        
        // Reconstruct the correct name and namespace
        let mut new_dependencies: HashSet<Dependency> = HashSet::new();
        for dependency in package.access_dependencies().get_all() {
            let new_dependency: Dependency =
                Dependency::new(dependency.url.clone(), dependency.version.clone()).unwrap();
            new_dependencies.insert(new_dependency);
        }
        package.replace_dependencies(Dependencies::new(new_dependencies));

        package
    }
}

impl Package {
    pub fn new(name: String, is_library: bool, interpreter: ShellType) -> Self {
        let mut entrypoint: String = String::from(DEFAULT_EXECUTABLE_ENTRYPOINT);
        if is_library {
            entrypoint = String::from(DEFAULT_LIBRARY_ENTRYPOINT);
        }
        
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
            String::from(DEFAULT_LIBRARY_ENTRYPOINT)
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
        let mut package_json_path: PathBuf = file.to_path_buf();
        if !file.ends_with(DEFAULT_PACKAGE_JSON) {
            package_json_path = file.join(DEFAULT_PACKAGE_JSON);
        }
        if !package_json_path.exists() {
            return Err(anyhow!(
                "{} is missing in the provided package path",
                DEFAULT_PACKAGE_JSON
            ));
        }
        
        let package_path: PathBuf = package_json_path.parent().unwrap().to_path_buf();
        
        let file: File = File::open(&package_json_path)?;
        let package_json: Package = Package::from(file);
        verify_package_integrity(&package_path, &package_json)?;

        Ok(package_json)
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

    pub fn access_dependencies(&mut self) -> &mut Dependencies {
        &mut self.dependencies
    }

    pub fn replace_dependencies(&mut self, dependencies: Dependencies) {
        self.dependencies = dependencies;
    }
}

/// Checks if a given directory contains a `package.json` file, indicating it is a package.
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

pub fn verify_package_integrity(
    package_path: &Path,
    package_json: &Package,
) -> Result<(), Error> {
    let entrypoint: PathBuf = package_path.join(&package_json.entrypoint);
    if !entrypoint.is_file() {
        return Err(anyhow!(
            "Entrypoint {} is missing in the provided package path",
            entrypoint.display()
        ));
    }

    let setup_script: PathBuf = package_path.join(&package_json.install.setup_script);
    if !setup_script.is_file() {
        return Err(anyhow!(
            "Setup script {} is missing in the provided package path",
            setup_script.display()
        ));
    }

    let uninstall_script: PathBuf = package_path.join(&package_json.uninstall);
    if !uninstall_script.is_file() {
        return Err(anyhow!(
            "Uninstall script {} is missing in the provided package path",
            uninstall_script.display()
        ));
    }
    
    let dependencies_folder: PathBuf = package_path.join(DEFAULT_DEPENDENCIES_FOLDER);
    if !dependencies_folder.is_dir() {
        return Err(anyhow!(
            "Dependencies folder {} is missing in the provided package path",
            dependencies_folder.display()
        ));
    }

    let src_folder: PathBuf = package_path.join(DEFAULT_SRC_FOLDER);
    if !src_folder.is_dir() {
        return Err(anyhow!(
            "Src folder {} is missing in the provided package path",
            src_folder.display()
        ));
    }

    Ok(())
}
