use anyhow::{Error, Result, anyhow};
use std::io::Write;
use std::path::Path;

use super::metadata::Package;
use super::std_lib::create_std_library;
use crate::properties::{DEFAULT_PACKAGE_JSON, DEFAULT_SRC_FOLDER};

/// Creates the package directory structure and necessary files
pub fn create_package_structure(path_to_package: &Path, package: &Package) -> Result<(), Error> {
    if !path_to_package.is_dir() {
        return Err(anyhow!(
            "A shell script project must be initialized in a directory!"
        ));
    }

    // Create a `src` folder
    std::fs::create_dir(path_to_package.join(DEFAULT_SRC_FOLDER))?;

    // Create all the necessary files
    create_entrypoint_script(path_to_package, package)?;
    create_package_json(path_to_package, package)?;
    create_setup_script(path_to_package, package)?;
    create_uninstall_script(path_to_package, package)?;

    // Create standard library directory and include function
    create_std_library(path_to_package, package)?;

    // Create dependencies directory
    std::fs::create_dir(path_to_package.join("dependencies"))?;

    Ok(())
}

/// Creates the main entrypoint script for the package
pub fn create_entrypoint_script(path_to_package: &Path, package: &Package) -> Result<(), Error> {
    // Get the shebang based on the interpreter set in `package.json`
    let shebang: &str = package.interpreter.get_shebang();

    // Create entrypoint script (either main.sh or lib.sh) based on whether it's a library
    let script_content: String;
    let script_filename: &str;

    if package.is_library {
        // Library script content
        script_filename = &package.entrypoint;
        script_content = format!(
            "{}\n\n# This is a library package\n# Define your functions below\n\n# Include function for dependency management\n. \"./src/std/include.sh\"\n\ngreet() {{\n    echo \"Hello from the library!\"\n}}\n",
            shebang
        );
    } else {
        // Main script content
        script_filename = &package.entrypoint;
        script_content = format!(
            "{}\n\n# Include standard library functions\n. \"./src/std/include.sh\"\n\nmain() {{\n    echo \"Hello World!\"\n}}\n\nmain \"$@\"",
            shebang
        );
    }

    // Create the entrypoint script
    match std::fs::File::create_new(path_to_package.join(script_filename)) {
        Ok(mut file) => {
            file.write_all(script_content.as_bytes())?;
            // Make sure the script is executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = file.metadata()?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755); // rwxr-xr-x
                std::fs::set_permissions(path_to_package.join(script_filename), permissions)?;
            }
        }
        Err(_) => {
            return Err(anyhow!(
                "A `{}` file already exists in this directory. Please remove or rename it before proceeding!",
                script_filename
            ));
        }
    };

    Ok(())
}

/// Creates the package.json file
pub fn create_package_json(path_to_package: &Path, package: &Package) -> Result<(), Error> {
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

    Ok(())
}

/// Creates the setup script
pub fn create_setup_script(path_to_package: &Path, package: &Package) -> Result<(), Error> {
    let shebang: &str = package.interpreter.get_shebang();
    let setup_script_content: &String = &package.install.setup_script;

    match std::fs::File::create_new(path_to_package.join(setup_script_content)) {
        Ok(mut file) => {
            file.write_all(
                format!("{}\n\necho \"Setting up the package...\"\n\n# Install dependencies if any\nif [ -d \"./dependencies\" ]; then\n    for dep in ./dependencies/*; do\n        if [ -f \"$dep/install.sh\" ]; then\n            echo \"Installing dependency: $(basename \"$dep\")\"\n            (cd \"$dep\" && ./install.sh)\n        fi\n    done\nfi", shebang).as_bytes(),
            )?;

            // Make sure the script is executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = file.metadata()?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755); // rwxr-xr-x
                std::fs::set_permissions(path_to_package.join(setup_script_content), permissions)?;
            }
        }
        Err(_) => {
            return Err(anyhow!(
                "A setup script file already exists in this directory. Please remove or rename it before proceeding!"
            ));
        }
    };

    Ok(())
}

/// Creates the uninstall script
pub fn create_uninstall_script(path_to_package: &Path, package: &Package) -> Result<(), Error> {
    let shebang: &str = package.interpreter.get_shebang();
    let uninstall_script_content: &String = &package.uninstall;

    match std::fs::File::create_new(path_to_package.join(uninstall_script_content)) {
        Ok(mut file) => {
            file.write_all(
                format!("{}\n\necho \"Uninstalling the package...\"\n\n# Run uninstall scripts for dependencies if any\nif [ -d \"./dependencies\" ]; then\n    for dep in ./dependencies/*; do\n        if [ -f \"$dep/uninstall.sh\" ]; then\n            echo \"Uninstalling dependency: $(basename \"$dep\")\"\n            (cd \"$dep\" && ./uninstall.sh)\n        fi\n    done\nfi", shebang).as_bytes(),
            )?;

            // Make sure the script is executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = file.metadata()?;
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755); // rwxr-xr-x
                std::fs::set_permissions(
                    path_to_package.join(uninstall_script_content),
                    permissions,
                )?;
            }
        }
        Err(_) => {
            return Err(anyhow!(
                "An uninstall script file already exists in this directory. Please remove or rename it before proceeding!"
            ));
        }
    };

    Ok(())
}
