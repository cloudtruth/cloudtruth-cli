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
            interactive: cli.is_interactive(),
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

impl Cli {
    pub fn is_interactive(&self) -> bool {
        !self.non_interactive
            && (self.interactive
                || !is_ci() && std::io::stdin().is_terminal() && std::io::stdout().is_terminal())
    }
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

/// Helper to detect common CI environment variables
fn is_ci() -> bool {
    macro_rules! check_ci_vars {
        ($name:literal $(,$names:literal)*$(,)?) => { std::env::var_os($name).is_some() $(|| std::env::var_os($names).is_some())* }
    }
    /// List is from watson/ci-info
    static IS_CI: OnceCell<bool> = OnceCell::new();
    *IS_CI.get_or_init(|| {
        check_ci_vars!(
            "CI",
            "BUILD_ID",
            "BUILD_NUMBER",
            "CI_APP_ID",
            "CI_BUILD_ID",
            "CI_BUILD_NUMBER",
            "CI_NAME",
            "CONTINUOUS_INTEGRATION",
            "RUN_ID",
            "CIRCLECI",
            "GITLAB_CI",
            "APPVEYOR",
            "DRONE",
            "MAGNUM",
            "SEMAPHORE",
            "JENKINS_URL",
            "bamboo_planKey",
            "TF_BUILD",
            "TEAMCITY_VERSION",
            "BUILDKITE",
            "HUDSON_URL",
            "GO_PIPELINE_LABEL",
            "BITBUCKET_COMMIT",
            "GITHUB_ACTIONS",
        )
    })
}
