use cargo_swift::*;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
enum Action {
    #[command()]
    /// Initializes a new Rust project that can be packaged as Swift package
    ///
    /// Generates boilerplate code for setting up dependencies and bridge modules
    Init {
        #[arg(index = 1)]
        crate_name: String,
    },

    #[command()]
    /// Packages the Rust crate in the current directory as Swift package
    ///
    Package {
        #[arg(short, long, trailing_var_arg = true, num_args = 1..=4, ignore_case = true)]
        platforms: Option<Vec<package::Platform>>,
        #[arg(short = 'n', long = "name")]
        package_name: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.action {
        Action::Init { crate_name } => init::run(crate_name),

        Action::Package {
            platforms,
            package_name,
        } => package::run(platforms, package_name),
    }
}
