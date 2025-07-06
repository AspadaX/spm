# SPM (Shell Program Manager) - Project Structure

## Overview

SPM is a Shell Program Manager written in Rust. It allows users to create, install, manage, and run individual shell script programs (.sh files) across different platforms and shell interpreters.

## Project Metadata

- **Name**: spm
- **Version**: 0.1.43
- **Edition**: 2024
- **License**: MIT
- **Repository**: https://github.com/aspadax/spm
- **Author**: Xinyu Bao <baoxinyuworks@163.com>

## Directory Structure

```
spm/
├── .github/
│   └── workflows/
│       └── release.yml          # GitHub Actions CI/CD pipeline for releases
├── .gitignore                   # Git ignore patterns
├── Cargo.lock                  # Dependency lock file
├── Cargo.toml                  # Rust project configuration and dependencies
├── Changes.md                  # Changelog (currently empty)
├── LICENSE                     # MIT license file
├── README.md                   # Project documentation and usage guide
├── dist-workspace.toml         # Distribution workspace configuration
└── src/                        # Source code directory
    ├── arguments.rs            # Command-line argument parsing and definitions
    ├── display_control.rs      # User interface and message display utilities
    ├── main.rs                 # Application entry point and command routing
    ├── program.rs              # Program management core functionality
    ├── properties.rs           # Configuration constants and default values
    ├── shell.rs                # Shell interpreter abstraction and execution
    └── utilities.rs            # Helper functions and utility operations
```

## Core Components

### 1. Main Application (`main.rs`)
- **Purpose**: Application entry point and command dispatcher
- **Key Functions**:
  - Command-line argument parsing using Clap
  - Program manager initialization
  - Command routing to appropriate handlers
  - PATH validation for binary directory

### 2. Command Line Interface (`arguments.rs`)
- **Purpose**: Defines all CLI commands and their arguments
- **Supported Commands**:
  - `run` - Execute shell scripts or programs
  - `install` - Install programs from local files or Git repositories
  - `list` - Display installed programs
  - `uninstall` - Remove installed programs
  - `check` - Validate shell script syntax (planned)
  - `new` - Create new shell script files with sh shebang
  - `version` - Display version information

### 3. Program Management (`program.rs`)
- **Purpose**: Core program management functionality
- **Key Components**:
  - `Program` struct: Unified structure representing shell script programs with metadata
  - `ProgramManager` struct: Handles program operations
- **Features**:
  - Program creation and installation from local files
  - Git repository cloning and batch installation
  - Program search and keyword matching
  - Metadata extraction from shell scripts
  - Interpreter detection from shebang lines
  - Recursive directory scanning for shell scripts

### 4. Shell Abstraction (`shell.rs`)
- **Purpose**: Multi-shell support and script execution
- **Supported Shells**:
  - `sh` (default for Unix-like systems)
  - `bash` (Bourne Again Shell)
  - `zsh` (Z Shell)
  - `cmd` (Windows Command Prompt)
- **Features**:
  - Cross-platform script execution
  - Execution context management (script directory vs. current directory)
  - Shebang generation for different interpreters

### 5. Utilities (`utilities.rs`)
- **Purpose**: Helper functions and common operations
- **Key Features**:
  - Program search and execution logic
  - Git repository cloning and authentication
  - PATH environment variable validation
  - Environment setup for shell configuration
  - Display formatting for program listings
  - Temporary directory management

### 6. Display Control (`display_control.rs`)
- **Purpose**: User interface and message formatting
- **Features**:
  - Colored console output using the `console` crate
  - Table formatting for program listings
  - Message levels (Logging, Error, Warn, Input)
  - Interactive user input handling

### 7. Configuration (`properties.rs`)
- **Purpose**: Application constants and default values
- **Constants**:
  - `.spm` - Default SPM folder name
  - `programs` - Default programs subdirectory
  - `tmp` - Temporary folder name

## Dependencies

### Core Dependencies
- **anyhow** (1.0.98) - Error handling and context
- **clap** (4.5.27) - Command-line argument parsing
- **console** (0.15.11) - Terminal styling and colors
- **dirs** (6.0.0) - Platform-specific directory paths
- **git2** (0.20.1) - Git repository operations
- **prettytable** (0.10.0) - Table formatting for output
- **serde** (1.0.219) - Serialization/deserialization
- **serde_json** (1.0.140) - JSON handling
- **which** (7.0.3) - Executable path resolution
- **auth-git2** (0.5.7) - Git authentication

### System Dependencies
- **openssl-sys** (0.9) - SSL/TLS support (vendored)
- **libz-sys** (1.1) - Compression library (static)

### Development Dependencies
- **tempfile** (3.17.1) - Temporary file handling for tests

## Program Structure

SPM manages individual shell script programs with the following characteristics:
- Each program is a single `.sh` file
- Programs are stored directly in the programs directory
- No additional metadata files required
- Programs can be executed directly by name
- Automatic interpreter detection based on shebang line

### Example Program Structure
```bash
#!/bin/bash
# This is a simple shell program managed by SPM

echo "Hello from my shell program!"
```

## Installation and Storage

- **User Directory**: `~/.spm/`
- **Programs Directory**: `~/.spm/programs/`
- **Binary Directory**: `~/.spm/bin/`
- **Temporary Directory**: `~/.spm/tmp/`

### Program Installation Process
1. Validate shell script file
2. Copy program to `~/.spm/programs/`
3. Create symbolic links in `~/.spm/bin/` for executables
4. Update PATH environment variable if needed

## CI/CD Pipeline

The project uses GitHub Actions for automated releases:
- **Workflow**: `.github/workflows/release.yml`
- **Tool**: cargo-dist for cross-platform distribution
- **Triggers**: Git tags matching version patterns
- **Artifacts**: Platform-specific binaries and installers
- **Platforms**: Linux, macOS, Windows

## Development Features

### Architecture Patterns
- **Error Handling**: Consistent use of `anyhow::Result` for error propagation
- **Configuration**: Centralized constants in `properties.rs`
- **Modularity**: Clear separation of concerns across modules
- **Cross-platform**: Conditional compilation for different operating systems
- **User Experience**: Rich terminal output with colors and formatting

## Usage Patterns

### Creating a New Program
```bash
spm new my_program          # Create new program file with sh shebang
```

### Installing Programs
```bash
spm install ./my_program.sh                    # Install local program
spm install https://github.com/user/repo.git   # Install all scripts from Git repository
```

### Managing Programs
```bash
spm list                    # List installed programs
spm uninstall program_name  # Remove program
spm run program_name        # Execute program
spm run ./script.sh         # Execute local script
```

### Development Commands
```bash
spm check ./script.sh       # Validate shell script syntax
spm version                 # Show version information
```

## Key Changes from Package-based to Program-based

1. **Simplified Structure**: No more `package.json` files or complex directory structures
2. **Direct File Management**: Each program is a single `.sh` file
3. **Automatic Metadata**: Program information is extracted from the script itself
4. **Streamlined Installation**: Direct file copying instead of directory management
5. **Reduced Complexity**: Eliminated Git repository support and namespace management
6. **Focus on Simplicity**: Easy-to-use shell script management without overhead