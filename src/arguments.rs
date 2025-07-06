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
    /// Install a shell script program
    Install(InstallArguments),
    /// Show installed shell script programs
    List(ListArguments),
    /// Uninstall shell script programs
    #[clap(short_flag = 'r')]
    Uninstall(UninstallArguments),
    /// Validate the shell script syntax
    Check(CheckArguments),
    /// Create a new shell script program
    New(NewArguments),
    /// Check version info
    #[clap(short_flag = 'v')]
    Version(VersionArguments),
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(false))]
pub struct RunArguments {
    /// A path to a shell script, or keyword(s) of a shell script.
    /// Single keyword: `spm run keyword1`.
    /// Multiple keywords: `spm run "keyword1 keyword2"`.
    #[arg(group = "sources", default_value = ".")]
    pub expression: String,

    /// Additional arguments to pass to the shell script
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(true))]
pub struct InstallArguments {
    /// Path to your shell script program, or a url to a shell script program git repository
    #[arg(group = "sources")]
    pub path: String,
    /// Force to install the program, or perform an update. Use `-F` for short.
    #[arg(short = 'F', long, group = "sources", default_value_t = false)]
    pub force: bool,
    /// Specify a base url if you would like to install a program hosted in
    /// a differet git repository other than GitHub.
    /// Use `-u` for short.
    #[arg(
        short = 'u',
        long,
        group = "sources",
        default_value = "https://github.com"
    )]
    pub base_url: String,
}

#[derive(Debug, Parser)]
pub struct ListArguments;

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct UninstallArguments {
    /// Index to your shell script in the bookmark.
    /// Can be obtained with `spm list`
    #[arg(group = "sources")]
    pub expression: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct CheckArguments {
    /// A path to a shell script, or a shell script program
    #[arg(group = "sources")]
    pub expression: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct NewArguments {
    /// Name the generated shell script
    #[arg(group = "sources")]
    pub name: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(false))]
pub struct VersionArguments;
