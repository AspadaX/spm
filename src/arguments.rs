use clap::{
    Args, Parser, Subcommand,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
    crate_authors, crate_description, crate_version,
};

// Configures Clap v3-style help menu colors
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Debug, Parser)]
#[command(name = "spm", author = crate_authors!(), long_version = crate_version!())]
#[command(about = crate_description!())]
#[command(styles = STYLES)]
pub struct Arguments {
    /// Groupped features provided by `spm`
    #[clap(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a shell script
    Run(RunArguments),
    /// Install a shell script package
    Install(InstallArguments),
    /// Show installed shell script packages
    List(ListArguments),
    /// Uninstall shell script packages
    #[clap(short_flag = 'r')]
    Uninstall(UninstallArguments),
    /// Validate the shell script syntax
    Check(CheckArguments),
    /// Create a new shell script project
    New(NewArguments),
    /// Create a new shell script project under the current working directory
    Init(InitializeArguments),
    /// Check version info
    #[clap(short_flag = 'v')]
    Version(VersionArguments),
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct RunArguments {
    /// Index of the shell script, or a path to a shell script, or keyword(s) of a shell script.
    /// Single keyword: `spm run keyword1`.
    /// Multiple keywords: `spm run "keyword1 keyword2"`.
    #[arg(group = "sources")]
    pub expression: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct InstallArguments {
    /// Path to your shell script project, or a url to a shell script project git repository
    #[arg(group = "sources")]
    pub path: String,
}

#[derive(Debug, Parser)]
pub struct ListArguments;

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct UninstallArguments {
    /// Index to your shell script in the bookmark.
    /// Can be obtained with `spm list`
    #[arg(group = "sources")]
    pub index: Option<usize>,
    /// Completely reset the bookmark. This is useful
    /// when `spm` breaks.
    #[arg(short, long, group = "sources", default_value = "false")]
    pub reset: bool,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct CheckArguments {
    /// A path to a shell script, or a shell script project
    #[arg(group = "sources")]
    pub expression: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(true))]
pub struct NewArguments {
    /// Name the generated shell script, by default,
    /// it will be a template file.
    #[arg(group = "sources")]
    pub name: String,
    // If specified, the project will be a library.
    // Otherwise, it is an executable program.
    #[arg(short, long, group = "sources", default_value_t = false)]
    pub lib: bool,
    // /// If specified, spm will generate a shell script by using a LLM provided
    // /// in the environment variables
    // #[arg(short, long, group = "sources")]
    // pub prompt: Option<String>,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(true))]
pub struct InitializeArguments {
    // If specified, the project will be a library.
    // Otherwise, it is an executable program.
    #[arg(short, long, group = "sources", default_value_t = false)]
    pub lib: bool,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(false))]
pub struct VersionArguments;
