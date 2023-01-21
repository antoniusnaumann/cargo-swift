use std::time::Duration;

use cargo_swift::*;
use cargo_toml::Manifest;
use clap::{Parser, Subcommand, ValueEnum};
use execute::Execute;
use indicatif::*;
use swift_bridge_build::ApplePlatform;

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
        platforms: Option<Vec<Platform>>,
        #[arg(short = 'n', long = "name")]
        package_name: Option<String>,
    },
}

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value()]
enum Platform {
    Macos,
    Ios,
    Tvos,
    Watchos,
}

impl Platform {
    fn into_apple_platforms(self) -> Vec<ApplePlatform> {
        match self {
            Platform::Macos => vec![ApplePlatform::MacOS],
            Platform::Ios => vec![ApplePlatform::IOS, ApplePlatform::Simulator],
            Platform::Tvos => vec![ApplePlatform::TvOS],
            Platform::Watchos => vec![ApplePlatform::WatchOS],
        }
    }
}

fn main() {
    let args = Args::parse();

    if let Action::Init { crate_name } = args.action {
        println!("Init: {crate_name}");
        return;
    }

    if let Action::Package {
        platforms,
        package_name,
    } = args.action
    {
        println!("Package");
        // TODO: Allow path as optional argument to take other directories than current directory
        let manifest = Manifest::from_path("./Cargo.toml")
            .expect("Could not find Cargo.toml in this directory!");

        let platforms = platforms.unwrap_or_else(|| todo!("TODO: Interactive prompt!"));
        // TODO: Prompt this but suggest default name based on crate name
        let package_name = package_name.unwrap_or_else(|| todo!("Prompt!"));
        let crate_name = dbg!(manifest.package.unwrap().name.to_lowercase());

        if platforms.is_empty() {
            eprintln!("At least 1 platform needs to be selected!");
            return;
        }

        let targets: Vec<_> = platforms
            .into_iter()
            .flat_map(|p| p.into_apple_platforms())
            .map(|p| p.target())
            .collect();

        let tick_rate = Duration::from_millis(30);

        let spinner_style =
            ProgressStyle::with_template(" {spinner:.bold.dim}   {wide_msg}").unwrap();
        let spinner_finish_style =
            ProgressStyle::with_template("{prefix:.bold.green} {wide_msg}").unwrap();

        let sub_spinner_style = ProgressStyle::with_template("\t{msg}").unwrap();
        let sub_spinner_finish_style = ProgressStyle::with_template("\t{msg:.dim}").unwrap();

        for target in targets {
            let multi = MultiProgress::new();
            let spinner = multi.add(
                ProgressBar::new_spinner()
                    .with_style(spinner_style.clone())
                    .with_message(format!("Building target {}...", target.display_name())),
            );
            spinner.enable_steady_tick(tick_rate);
            for mut command in target.commands(&crate_name) {
                let sub = multi.add(
                    ProgressBar::new_spinner()
                        .with_style(sub_spinner_style.clone())
                        .with_message(command.info()),
                );
                sub.enable_steady_tick(tick_rate);

                command.execute().expect(
                    format!("Failed to execute build command: {}", command.info()).as_str(),
                );

                sub.set_style(sub_spinner_finish_style.clone());
                sub.finish();
            }
            spinner.set_style(spinner_finish_style.clone());
            spinner.set_prefix("DONE");
            spinner.finish_with_message(format!(
                "Successfully built target {}!",
                target.display_name()
            ))
        }

        return;
    }
}
