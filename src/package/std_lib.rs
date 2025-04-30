use anyhow::{Error, Result};
use std::io::Write;
use std::path::Path;

use crate::properties::{DEFAULT_SRC_FOLDER, DEFAULT_STANDARD_LIBRARY_FOLDER};

use super::metadata::Package;

/// Creates the standard library for a shell script package
pub fn create_std_library(path_to_package: &Path, package: &Package) -> Result<(), Error> {
    // Create the std directory under src
    let std_dir = path_to_package
        .join(DEFAULT_SRC_FOLDER)
        .join(DEFAULT_STANDARD_LIBRARY_FOLDER);
    std::fs::create_dir_all(&std_dir)?;

    // Create the include.sh file with include function
    create_include_function(&std_dir, package)?;

    Ok(())
}

/// Creates the include.sh file with include function
fn create_include_function(std_dir: &Path, package: &Package) -> Result<(), Error> {
    let shebang = package.interpreter.get_shebang();
    let include_path = std_dir.join("include.sh");

    // When using raw strings with format, we need to double the curly braces for shell script
    let include_content = format!(
        "{}\n\n# Shell Package Manager include function\n# This function allows importing dependencies and other shell scripts\n\n# Global variable to track already included files to avoid duplicate inclusions\nif [ -z \"$SPM_INCLUDED_FILES\" ]; then\n    export SPM_INCLUDED_FILES=\"\"\nfi\n\n# Include a dependency or local file\n# Usage: include \"dependency_name\" [file_path]\n# Examples:\n#   include \"logger\"                # Include main entry point of logger dependency\n#   include \"logger\" \"src/utils.sh\" # Include specific file from logger dependency\n#   include \"./src/helpers.sh\"      # Include local file\ninclude() {{\n    local dependency=\"\"\n    local file_path=\"\"\n    \n    # Check if this is a dependency include or a local file include\n    if [ $# -eq 1 ]; then\n        # Single argument could be a dependency name or a local file\n        if [ -d \"./dependencies/$1\" ]; then\n            # It's a dependency - use its main entry point\n            dependency=\"$1\"\n            local package_json=\"./dependencies/$dependency/package.json\"\n            if [ -f \"$package_json\" ]; then\n                # Extract the entrypoint from package.json\n                if command -v jq >/dev/null 2>&1; then\n                    file_path=\"./dependencies/$dependency/$(jq -r '.entrypoint' \"$package_json\")\"\n                else\n                    # Fallback if jq is not available\n                    file_path=\"./dependencies/$dependency/main.sh\"\n                    if [ ! -f \"$file_path\" ]; then\n                        file_path=\"./dependencies/$dependency/lib.sh\"\n                    fi\n                fi\n            else\n                file_path=\"./dependencies/$dependency/main.sh\"\n                if [ ! -f \"$file_path\" ]; then\n                    file_path=\"./dependencies/$dependency/lib.sh\"\n                fi\n            fi\n        else\n            # It's a local file\n            file_path=\"$1\"\n        fi\n    elif [ $# -eq 2 ]; then\n        # Two arguments: dependency and specific file\n        dependency=\"$1\"\n        file_path=\"./dependencies/$dependency/$2\"\n    else\n        echo \"Error: include requires 1 or 2 arguments\" >&2\n        return 1\n    fi\n    \n    # Convert to absolute path to ensure uniqueness check works\n    local abs_path=\"$(cd \"$(dirname \"$file_path\")\" 2>/dev/null && pwd)/$(basename \"$file_path\")\"\n    \n    # Check if the file has already been included\n    if echo \"$SPM_INCLUDED_FILES\" | grep -q \"$abs_path\"; then\n        return 0\n    fi\n    \n    # Check if the file exists\n    if [ ! -f \"$file_path\" ]; then\n        echo \"Error: Cannot include '$file_path' - file not found\" >&2\n        return 1\n    fi\n    \n    # Add to the list of included files\n    SPM_INCLUDED_FILES=\"$SPM_INCLUDED_FILES:$abs_path\"\n    \n    # Source the file\n    . \"$file_path\"\n    return $?\n}}",
        shebang
    );

    let mut file = std::fs::File::create(&include_path)?;
    file.write_all(include_content.as_bytes())?;

    // Make the file executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = file.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755); // rwxr-xr-x
        std::fs::set_permissions(&include_path, permissions)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::ShellType;
    use std::fs;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_create_std_library() {
        // Create a temporary directory for testing
        let temporary_dir = tempdir().unwrap();
        let package_path = temporary_dir.path();

        // Create src directory since it's expected by create_std_library
        fs::create_dir(package_path.join(DEFAULT_SRC_FOLDER)).unwrap();

        // Create a simple package
        let package =
            super::super::metadata::Package::new("test-package".to_string(), false, ShellType::Sh);

        // Create the standard library
        let result = create_std_library(package_path, &package);
        assert!(result.is_ok());

        // Verify the std directory was created
        let std_dir = package_path
            .join(DEFAULT_SRC_FOLDER)
            .join(DEFAULT_STANDARD_LIBRARY_FOLDER);
        assert!(std_dir.exists());
        assert!(std_dir.is_dir());

        // Verify the include.sh file was created
        let include_file = std_dir.join("include.sh");
        assert!(include_file.exists());
        assert!(include_file.is_file());

        // Read the file contents to verify it contains the include function
        let mut file_content = String::new();
        let mut file = fs::File::open(include_file).unwrap();
        file.read_to_string(&mut file_content).unwrap();

        assert!(file_content.contains("include()"));
        assert!(file_content.contains("SPM_INCLUDED_FILES"));
    }
}
