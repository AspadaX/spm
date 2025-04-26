use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs::{DirEntry, File};
use std::path::{Path, PathBuf};

use super::dependency::Dependencies;
use crate::properties::{
    DEFAULT_EXECUTABLE_ENTRYPOINT, DEFAULT_LIBRARY_ENTRYPOINT, DEFAULT_PACKAGE_JSON,
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
        let package: Package =
            serde_json::from_reader(value).expect("Failed to parse JSON file into Package");
        package
    }
}

impl Package {
    pub fn new(name: String, is_library: bool, interpreter: ShellType) -> Self {
        let entrypoint: String = if is_library {
            String::from(DEFAULT_LIBRARY_ENTRYPOINT)
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
        let package_json_path: PathBuf = file.join(DEFAULT_PACKAGE_JSON);
        if !package_json_path.is_file() {
            return Err(anyhow!(
                "{} is missing in the provided package path",
                DEFAULT_PACKAGE_JSON
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

    pub fn access_dependencies(&mut self) -> &mut Dependencies {
        &mut self.dependencies
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
