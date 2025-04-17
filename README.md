# spm, Shell Package Manager

Shell Package Manager written in Rust with native AI support. 

Notice: This package is still in its early development phase. Be sure to check this out later! Or try it now!

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
Shell Package Manager written in Rust with native AI support.

Usage: spm <COMMAND>

Commands:
  run            Run a shell script
  install        Install a shell script package
  list           Show installed shell script packages
  uninstall, -r  Uninstall shell script packages
  check          Validate the shell script syntax
  new            Create a new shell script project
  init           Create a new shell script project under the current working directory
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

## Create a Shell Script Project
You can initialize a shell script project by using this command:
```bash
spm new <your-package-name>
```
Or, initialize a project inside a folder:
```bash
spm init
```
You will get the following files setup for you:
```bash
install.sh # A setup script to run when someone installs your project as a package
main.sh # The main entrypoint of your project. 
package.json # A descriptive file contains the project information
src # Source files
uninstall.sh # An uninstallation script to run when someone uninstalls your package
```

# TODOs

- [ ] Naming check for packages. Restricted to `*-*` format.
- [ ] Support `spm run` in a project directory for development. 
- [ ] Support `include()` function in every `spm` shell script projects. So that you can make a library for other shell script projects to use. 
- [ ] Support `dependencies`. 
- [ ] Support AI features. 
- [ ] Support install a package from a git repository. 

And more to go as you raise an `issue` in this repository! 

# Contribute

Any contribution is welcome. Issues, or PRs, whatever. There are no guidelines on how you should structure your code in this repository just yet. 

# License

This project is open source under MIT license. 
