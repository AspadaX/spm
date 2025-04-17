use std::path::Path;

use anyhow::{Error, Result};

use crate::{
    display_control::display_form, package::{is_inside_a_package, Package, PackageMetadata}, shell::execute_shell_script
};

pub fn execute_run_command(expression: String) -> Result<(), Error> {
    let path: &Path = Path::new(&expression);

    // Case 1: input is a shell script
    if path.is_file() {
        return execute_shell_script(&expression);
    }

    // Case 2: input is a shell script project/package
    if path.is_dir() {
        // Validate the directory
        if is_inside_a_package(path)? {
            let package = Package::from_file(path)?;
            let main_entrypoint_filename: &str = package.access_main_entrypoint();

            return execute_shell_script(
                &path
                    .join(main_entrypoint_filename)
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    // Case 3: Input is a keyword or keywords

    Ok(())
}

pub fn show_packages(packages_metadata: &Vec<PackageMetadata>) {
    let mut form_data: Vec<Vec<String>> = Vec::new();

    for (index, metadata) in packages_metadata.iter().enumerate() {
        form_data.push(vec![
            index.to_string(),
            metadata.get_pacakge_name().to_string(),
            metadata.get_description().to_string(),
            metadata.get_version().to_string(),
        ]);
    }

    display_form(vec!["Index", "Name", "Description", "Version"], &form_data);
}
