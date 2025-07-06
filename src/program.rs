use std::io::Write;
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use anyhow::{Error, Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::properties::{DEFAULT_SPM_FOLDER, DEFAULT_SPM_PROGRAMS_FOLDER};
use crate::shell::ShellType;

/// Represent a shell script program
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Program {
    // The name of the program (derived from filename)
    name: String,
    // The path to the program file (optional for new programs)
    path_to_program: Option<PathBuf>,
    // The interpreter used for this program
    interpreter: ShellType,
}

impl Program {
    pub fn new(name: String, interpreter: ShellType) -> Self {
        Self { 
            name, 
            path_to_program: None,
            interpreter 
        }
    }

    /// Create a Program from a .sh file path
    pub fn from_file(file_path: &Path) -> Result<Self, Error> {
        if !file_path.is_file() {
            return Err(anyhow!("The provided path is not a file"));
        }

        let file_name = file_path
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_string_lossy()
            .to_string();

        // Try to detect interpreter from shebang
        let interpreter = detect_interpreter_from_file(file_path).unwrap_or(ShellType::Sh);

        Ok(Self {
            name: file_name,
            path_to_program: Some(file_path.to_path_buf()),
            interpreter,
        })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_program_path(&self) -> Option<&str> {
        self.path_to_program.as_ref().map(|p| p.as_os_str().to_str().unwrap())
    }

    pub fn get_interpreter(&self) -> &ShellType {
        &self.interpreter
    }
}

#[derive(Debug, Clone)]
pub struct ProgramManager {
    root_directory: PathBuf,
}

impl ProgramManager {
    pub fn new() -> Result<Self, Error> {
        let root_directory: PathBuf = dirs::home_dir()
            .ok_or_else(|| anyhow!("Failed to locate home directory"))?
            .join(DEFAULT_SPM_FOLDER);

        if !root_directory.exists() {
            // Create the programs folder
            match std::fs::create_dir_all(&root_directory.join("programs")) {
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

        Ok(Self { root_directory })
    }

    /// Returns the path to the binary directory where executable scripts are symlinked.
    pub fn get_bin_directory(&self) -> Result<PathBuf, Error> {
        let bin_dir = self.root_directory.join("bin");

        // Create the bin directory if it doesn't exist
        if !bin_dir.exists() {
            std::fs::create_dir_all(&bin_dir)?;
        }

        Ok(bin_dir)
    }

    /// Retrieves a `Program` object by its name.
    pub fn get_program_by_name(&self, program_name: String) -> Result<Program, Error> {
        let installed_programs: Vec<Program> = self.get_installed_programs()?;

        // Look for exact program name match
        for program in installed_programs {
            if program.get_name() == program_name {
                return Ok(program);
            }
        }

        Err(anyhow!("Program with name '{}' not found", program_name))
    }

    pub fn keyword_search(&self, keywords: &str) -> Result<Vec<Program>, Error> {
        let words: Vec<String> = keywords
            .split(",")
            .map(|keyword: &str| keyword.to_lowercase())
            .collect();
        let mut matched_programs: Vec<(Program, usize)> = Vec::new();

        if let Ok(programs) = self.get_installed_programs() {
            for program in programs {
                let program_name: String = program.get_name().to_lowercase();

                // If exactly matches the program name
                if program_name == keywords.to_lowercase() {
                    matched_programs.push((program.clone(), 2)); // Higher score for exact match
                    continue;
                }

                let mut match_score = 0;

                for word in words.iter() {
                    // Skip if the keyword is empty
                    if word.is_empty() {
                        continue;
                    }

                    // When a keyword is found in the name
                    if program_name.contains(word) {
                        match_score += 1;
                    }
                }

                // Add the program with its match score if any matches found
                if match_score > 0 {
                    matched_programs.push((program.clone(), match_score));
                }
            }
        }

        // Sort the programs by match count in descending order
        matched_programs.sort_by(|a, b| b.1.cmp(&a.1));

        let mut results: Vec<Program> = Vec::new();
        for matched_program in matched_programs {
            // Skip the programs if the score is zero
            if matched_program.1 != 0 {
                results.push(matched_program.0);
            }
        }

        Ok(results)
    }

    /// Returns the path to the program installation directory.
    pub fn access_program_installation_directory(&self) -> PathBuf {
        self.root_directory.join("programs")
    }

    /// Create a new shell script program file.
    pub fn create_program(&self, path_to_program: &Path, program: &Program) -> Result<(), Error> {
        if path_to_program.is_dir() {
            return Err(anyhow!(
                "A shell script program must be a file, not a directory!"
            ));
        }

        // Get the shebang based on the interpreter
        let shebang: &str = program.interpreter.get_shebang();

        // Create the shell script content
        let script_content = format!(
            "{}\n\nmain() {{\n    echo \"Hello from {}!\"\n}}\n\nmain \"$@\"",
            shebang, program.name
        );

        // Create the shell script file
        match std::fs::File::create_new(path_to_program) {
            Ok(mut file) => {
                file.write_fmt(format_args!("{}", script_content))?;
                // Make the file executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = file.metadata()?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(path_to_program, perms)?;
                }
            }
            Err(_) => {
                return Err(anyhow!(
                    "A file with this name already exists. Please choose a different name!"
                ));
            }
        };

        Ok(())
    }

    /// Retrieves the list of installed programs by scanning the program installation directory.
    pub fn get_installed_programs(&self) -> Result<Vec<Program>, Error> {
        let spm_dir: PathBuf = self.access_program_installation_directory();

        if !spm_dir.is_dir() {
            return Err(anyhow!(format!(
                "The program installation directory `~/{}/{}` does not exist",
                DEFAULT_SPM_FOLDER, DEFAULT_SPM_PROGRAMS_FOLDER
            )));
        }

        let mut installed_programs: Vec<Program> = Vec::new();

        // Read the programs directory
        for entry in std::fs::read_dir(spm_dir)? {
            let entry: DirEntry = entry?;
            let path: PathBuf = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "sh") {
                let program_name = path.file_stem().unwrap().to_string_lossy().to_string();

                let interpreter = detect_interpreter_from_file(&path).unwrap_or(ShellType::Sh);

                installed_programs.push(Program {
                    name: program_name,
                    path_to_program: Some(path),
                    interpreter,
                });
            }
        }

        Ok(installed_programs)
    }

    /// Installs a program by copying it to the program installation directory.
    pub fn install_program(&self, path_to_program: &Path, is_force: bool) -> Result<(), Error> {
        if !path_to_program.is_file() {
            return Err(anyhow!("The provided path must be a .sh file"));
        }

        if path_to_program.extension().map_or(true, |ext| ext != "sh") {
            return Err(anyhow!("Only .sh files are supported"));
        }

        let spm_dir: PathBuf = self.access_program_installation_directory();

        if !spm_dir.exists() {
            std::fs::create_dir_all(&spm_dir)?;
        }

        let program_name = path_to_program
            .file_name()
            .ok_or_else(|| anyhow!("Invalid program file name"))?;

        let destination = spm_dir.join(program_name);

        // Check if this program already exists
        if destination.exists() && !is_force {
            return Err(anyhow!(
                "The program already exists. Use `--force` (-F) flag to force an install or update"
            ));
        }

        // Copy the program file
        std::fs::copy(path_to_program, &destination)?;

        // Make sure the file is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&destination)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&destination, perms)?;
        }

        Ok(())
    }

    /// Installs all shell scripts from a Git repository.
    pub fn install_from_git(&self, git_url: &str, is_force: bool) -> Result<(), Error> {
        use crate::utilities::{create_temp_directory, cleanup_temp_repository, clone_git_repository};
        
        // Create temporary directory for cloning
        let temp_dir = create_temp_directory()?;
        let repo_path = temp_dir.join("repo");
        
        // Clone the repository
        clone_git_repository(git_url, &repo_path)?;
        
        // Find all .sh files in the repository
        let mut installed_count = 0;
        self.install_scripts_from_directory(&repo_path, is_force, &mut installed_count)?;
        
        // Cleanup temporary directory
        cleanup_temp_repository(&repo_path)?;
        
        if installed_count == 0 {
            return Err(anyhow!("No shell scripts found in the repository"));
        }
        
        Ok(())
    }
    
    /// Recursively install all .sh files from a directory.
    fn install_scripts_from_directory(&self, dir: &Path, is_force: bool, count: &mut usize) -> Result<(), Error> {
        if !dir.is_dir() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively search subdirectories
                self.install_scripts_from_directory(&path, is_force, count)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "sh") {
                // Install the shell script
                match self.install_program(&path, is_force) {
                    Ok(_) => {
                        *count += 1;
                        println!("Installed: {}", path.file_name().unwrap().to_string_lossy());
                    }
                    Err(e) => {
                        eprintln!("Failed to install {}: {}", path.file_name().unwrap().to_string_lossy(), e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Uninstalls a program by removing it from the installation directory.
    fn uninstall_program(&self, path_to_program: &Path) -> Result<(), Error> {
        if !path_to_program.exists() {
            return Err(anyhow!("The specified program path does not exist"));
        }

        std::fs::remove_file(path_to_program)
            .map_err(|e| anyhow!("Failed to remove program file: {}", e))?;

        Ok(())
    }

    pub fn uninstall_program_by_name(&self, program_name: String) -> Result<(), Error> {
        let program: Program = self.get_program_by_name(program_name)?;
        let program_path = program.get_program_path()
            .ok_or_else(|| anyhow!("Program path not available"))?;
        self.uninstall_program(Path::new(program_path))
    }
}

/// Detect the interpreter from the shebang line of a shell script file
fn detect_interpreter_from_file(file_path: &Path) -> Result<ShellType, Error> {
    let content = std::fs::read_to_string(file_path)?;
    let first_line = content.lines().next().unwrap_or("");

    if first_line.starts_with("#!") {
        if first_line.contains("bash") {
            return Ok(ShellType::Bash);
        } else if first_line.contains("zsh") {
            return Ok(ShellType::Zsh);
        } else if first_line.contains("cmd") {
            return Ok(ShellType::Cmd);
        } else if first_line.contains("sh") {
            return Ok(ShellType::Sh);
        }
    }

    // Default to sh if no shebang or unrecognized interpreter
    Ok(ShellType::Sh)
}

/// Normalize a program name
pub fn normalize_program_name(name: &str) -> String {
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
