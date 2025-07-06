# spm, Shell Program Manager

Shell Program Manager written in Rust for managing shell scripts. 

Notice: This program is still in its early development phase. Be sure to check this out later! Or try it now!

# Installation

## Cargo
If you have `cargo` installed, you could run the following command to set it up:
```bash
cargo install spm
```

## Shell Script
TODO

# Usage
Run the command to see a full list of available commands:
```bash
spm
```
The output should look like this:
```bash
Shell Program Manager written in Rust for managing shell scripts.

Usage: spm <COMMAND>

Commands:
  run            Run a shell script
  install        Install a shell script program or from a Git repository
  list           Show installed shell script programs
  uninstall, -r  Uninstall shell script programs
  check          Validate the shell script syntax
  new            Create a new shell script project
  version, -v    Check version info
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Run a Shell Script
You don't need to set the privilige if you use `spm` to run a shell script, just type:
```bash
spm run ./path/to/your/shell/script # Can be an absolute path too
```

## Create a Shell Script Program
You can create a shell script program by using this command:
```bash
spm new <your-program-name>
```
This will create a simple `.sh` file with a `sh` shebang and a "hello world" function structure.

## Install Programs from Git Repositories
You can now install all shell scripts from a Git repository:
```bash
spm install https://github.com/username/repository.git
```
This will clone the repository and install all `.sh` files found within it.

# TODOs

- [x] Support install a program from a git repository. 
- [ ] Support AI features.
- [ ] Check if the commands in a shell script has installed.

And more to go as you raise an `issue` in this repository! 

# Contribute

Any contribution is welcome. Issues, or PRs, whatever. There are no guidelines on how you should structure your code in this repository just yet. 

# License

This project is open source under MIT license.
