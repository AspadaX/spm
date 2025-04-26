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
    /// Install a shell script package globally
    Install(InstallArguments),
    /// Show installed shell script packages
    List(ListArguments),
    /// Uninstall shell script packages globally
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
    /// Add a dependency to the current package
    Add(AddDependencyArguments),
    /// Remove a dependency from the current package
    Remove(RemoveDependencyArguments),
    /// Refresh or update dependencies in the current package
    Refresh(RefreshArguments),
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
    /// Path to your shell script project, or a url to a shell script project git repository
    #[arg(group = "sources")]
    pub path: String,
    /// Force to install the package, or perform an update. Use `-F` for short.
    #[arg(short = 'F', long, group = "sources", default_value_t = false)]
    pub force: bool,
    /// Specify a base url if you would like to install a package hosted in
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
    // Specify the interpreter to use.
    // In UNIX-like system, `sh` is the default.
    // In Windows, `cmd` is the default.
    // Currently support: `sh`, `bash`, `zsh`, `cmd`.
    #[arg(short = 'I', long, group = "sources", default_value = "sh")]
    pub interpreter: String,
    // If specified, the project will be a library.
    // Otherwise, it is an executable program.
    #[arg(short, long, group = "sources", default_value_t = false)]
    pub lib: bool,
    // If specified, the package will be created under this namespace
    #[arg(short = 'n', long, group = "sources")]
    pub namespace: Option<String>,
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
    // Specify the interpreter to use.
    // In UNIX-like system, `sh` is the default.
    // In Windows, `cmd` is the default.
    // Currently support: `sh`, `bash`, `zsh`, `cmd`.
    #[arg(short = 'I', long, group = "sources", default_value = "sh")]
    pub interpreter: String,
    // If specified, the package will be created under this namespace
    #[arg(short = 'n', long, group = "sources")]
    pub namespace: Option<String>,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(false))]
pub struct VersionArguments;

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(true))]
pub struct AddDependencyArguments {
    /// Path to your shell script project, or a url to a shell script project git repository
    #[arg(group = "sources")]
    pub path: String,

    /// Version (branch, tag, or commit) to use
    #[arg(short, long, group = "sources", default_value = "main")]
    pub version: String,

    /// Specify a base url if you would like to install a package hosted in
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

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(true).multiple(false))]
pub struct RemoveDependencyArguments {
    /// Name of the dependency to remove
    #[arg(group = "sources")]
    pub name: String,
}

#[derive(Debug, Args)]
#[command(group = clap::ArgGroup::new("sources").required(false).multiple(true))]
pub struct RefreshArguments {
    /// Name of a specific dependency to update (if not specified, all dependencies will be updated)
    #[arg(short, long, group = "sources")]
    pub name: Option<String>,

    /// Namespace of the dependency (if needed to disambiguate)
    #[arg(short = 's', long)]
    pub namespace: Option<String>,

    /// Update to specific version (branch, tag, or commit)
    #[arg(short, long, group = "sources")]
    pub version: Option<String>,
}
