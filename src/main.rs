mod arguments;
mod shell;

use arguments::{Arguments, Commands};
use clap::Parser;
use shell::execute_shell_script;

fn main() {
    // Parse command line arguments
    let arguments = Arguments::parse();

    // Map the arguments to corresponding code logics
    match arguments.commands {
        Commands::Run(subcommand) => {
            execute_shell_script(&subcommand.expression);
        },
        Commands::Install(subcommand) => {},
        Commands::List(_) => {},
        Commands::Uninstall(subcommand) => {},
        Commands::Check(subcommand) => {},
        Commands::New(subcommand) => {},
        Commands::Initialize(_) => {},
        Commands::Version(_) => {}
    }
}
