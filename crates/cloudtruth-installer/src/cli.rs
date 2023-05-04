use clap::Parser;
use is_terminal::IsTerminal;
use once_cell::sync::OnceCell;

static GLOBALS: OnceCell<Globals> = OnceCell::new();

#[derive(Debug)]
struct Globals {
    verbose: bool,
    interactive: bool,
}

/// Global verbose flag.
/// Initialized after CLI parsing, and set to false otherwise.
pub fn verbose() -> bool {
    match GLOBALS.get() {
        Some(globals) => globals.verbose,
        _ => false,
    }
}

/// Global non-interactive flag. Indicates that we should not prompt the user for input.
/// Initialized after CLI parsing, and set to false otherwise.
pub fn interactive() -> bool {
    match GLOBALS.get() {
        Some(globals) => globals.interactive,
        _ => false,
    }
}

pub fn parse() -> Cli {
    let cli = Cli::parse();
    init_globals(&cli);
    cli
}

/// initialize global statics (verbosity, non-interactive, etc)
/// this funciton will panic if called twice
fn init_globals(cli: &Cli) {
    GLOBALS
        .set(Globals {
            verbose: cli.verbose,
            interactive: !cli.non_interactive
                && (cli.interactive
                    || (std::io::stdin().is_terminal() && std::io::stdout().is_terminal())),
        })
        .expect("CLI globals were initialized twice")
}

#[derive(Debug, clap::Parser)]
/// CloudTruth installer CLI
/// #[command(author, version, about, long_about)]
pub struct Cli {
    /// Subcommands
    #[command(subcommand)]
    command: Subcommand,
    /// Show verbose information
    #[arg(global = true, short, long, default_value_t = false)]
    verbose: bool,
    /// Force interactive mode, always prompt and ask for confirmations
    #[arg(global = true, short = 'i', long, overrides_with = "non_interactive")]
    interactive: bool,
    /// Force non-interactive mode, do not prompt or ask for confirmations
    #[arg(global = true, short = 'n', long, overrides_with = "interactive")]
    non_interactive: bool,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    #[command(about = "Install a Cloudtruth CLI ")]
    Install(InstallCommand),
}

#[derive(Debug, clap::Args)]
pub struct InstallCommand {
    /// Name of the program to install
    name: String,
    /// Version of the program to install (defaults to latest)
    version: Option<String>,
    #[command(flatten)]
    github_opts: GitHubOptions,
}

/// Options for GitHub API (for internal release workflows)
#[derive(Debug, clap::Args)]
#[group(multiple = true, required = false)]
pub struct GitHubOptions {
    #[arg(
        help_heading = "GitHub API Options (for internal CloudTruth release pipeline)",
        long,
        env = "CLOUDTRUTH_INSTALLER_GITHUB_AUTH_TOKEN",
        requires = "release_id"
    )]
    /// GitHub API Auth Token
    auth_token: Option<String>,
    #[arg(
        help_heading = "GitHub API Options (for internal CloudTruth release pipeline)",
        long,
        env = "CLOUDTRUTH_INSTALLER_GITHUB_RELEASE_ID",
        requires = "auth_token"
    )]
    /// GitHub API Release ID
    release_id: Option<String>,
}
