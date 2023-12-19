use std::process::ExitCode;

use cargo_swift::{init, package, Config, LibType, Mode};
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
    /// This is especially useful when invoked in an environment, where no user interaction is possible,
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

        #[arg(long, ignore_case = true, default_value_t = init::Vcs::Git)]
        vcs: init::Vcs,

        #[arg(long, ignore_case = true, default_value_t = LibType::Static)]
        lib_type: LibType,

        #[arg(short, long)]
        // Initialize the project without any explanatory boilerplate code
        plain: bool,

        #[arg(long = "macro")]
        // Initialize the project as a macro-only crate without .udl files
        macro_only: bool,
    },

    #[command()]
    /// Package Rust crate in current directory as Swift package
    ///
    Package {
        #[arg(short, long, trailing_var_arg = true, num_args = 1..=4, ignore_case = true)]
        platforms: Option<Vec<package::Platform>>,

        #[arg(short = 'n', long = "name")]
        package_name: Option<String>,

        #[arg(short, long)]
        /// Build package optimized for release (default: debug)
        release: bool,

        #[arg(long, ignore_case = true, default_value_t = package::LibTypeArg::Automatic)]
        /// Chose the how the library should be build. By default, this will be derived from the lib type provided in Cargo.toml
        lib_type: package::LibTypeArg,

        #[arg(long)]
        /// Disable warnings in generated Swift package code
        suppress_warnings: bool,

        #[arg(long)]
        /// Disable toolchains check
        skip_toolchains_check: bool,
    },
}

fn main() -> ExitCode {
    let Cargo::Swift(args) = Cargo::parse();
    let config = args.clone().into();

    let result = match args.action {
        Action::Init {
            crate_name,
            vcs,
            lib_type,
            plain,
            macro_only,
        } => init::run(crate_name, config, vcs, lib_type, plain, macro_only),

        Action::Package {
            platforms,
            package_name,
            suppress_warnings,
            release,
            lib_type,
            skip_toolchains_check,
        } => package::run(
            platforms,
            package_name,
            suppress_warnings,
            config,
            if release { Mode::Release } else { Mode::Debug },
            lib_type,
            skip_toolchains_check,
        ),
    };

    if let Err(e) = result {
        eprintln!("\n");
        eprintln!("Failed due to the following error: ");
        e.print();
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
