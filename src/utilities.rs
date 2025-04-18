use std::path::Path;

use anyhow::{anyhow, Error, Result};

use crate::{
    display_control::{display_form, display_message, display_tree_message, input_message, Level}, package::{is_inside_a_package, Package, PackageManager, PackageMetadata}, shell::execute_shell_script
};

pub fn execute_run_command(package_manager: &PackageManager, expression: String) -> Result<(), Error> {
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
    let package_candidates: Vec<PackageMetadata> = package_manager.keyword_search(&expression)?;
    // Throw an error if no chains are found
    if package_candidates.len() == 0 {
        return Err(anyhow!("No packages found"));
    }
    
    // Run the chain if it is exactly one
    if package_candidates.len() == 1 {
        return execute_shell_script(
            &path
                .join(package_candidates[0].get_main_entry_point())
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        );
    }
    
    display_message(Level::Logging, "Multiple packages found:");
    for (index, package_metadata) in package_candidates.iter().enumerate() {
        display_tree_message(1, &format!("{}: {}", index + 1, package_metadata.get_pacakge_name()));
    }
    let selection: usize = input_message("Please select a chain to execute:")?.trim().parse::<usize>()?;
    
    return execute_shell_script(
        &path
            .join(package_candidates[selection - 1].get_main_entry_point())
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );
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
