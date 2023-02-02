use cargo_swift::{init, package, Config};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum Cargo {
    Swift(Args),
}

#[derive(clap::Args, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,

    #[arg(short, long, global = true)]
    /// Silence all output except errors and interactive prompts
    silent: bool,

    #[arg(short = 'y', long, global = true)]
    /// Accept all default selections from all interactive prompts.
    ///
    /// This is especially useful when invoked in an environment,  where no user interaction is possible,
    /// e.g. a test runner. Prompts without a default state will be skipped as well, resulting in an error
    /// if the corresponding value was not set as an argument beforehand.
    accept_all: bool,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        Config {
            silent: args.silent,
            accept_all: args.accept_all,
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
enum Action {
    #[command()]
    /// Initialize a new Rust project that can be packaged as Swift package
    ///
    /// This command generates boilerplate code for setting up dependencies and bridge modules
    Init {
        #[arg(index = 1)]
        crate_name: String,
    },

    #[command()]
    /// Package Rust crate in current directory as Swift package
    ///
    Package {
        #[arg(short, long, trailing_var_arg = true, num_args = 1..=4, ignore_case = true)]
        platforms: Option<Vec<package::Platform>>,
        #[arg(short = 'n', long = "name")]
        package_name: Option<String>,
    },
}

fn main() {
    let Cargo::Swift(args) = Cargo::parse();
    let config = args.clone().into();

    match args.action {
        Action::Init { crate_name } => init::run(crate_name, config),

        Action::Package {
            platforms,
            package_name,
        } => package::run(platforms, package_name, config),
    }
}
