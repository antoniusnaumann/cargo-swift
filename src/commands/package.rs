use std::path::PathBuf;

use cargo_toml::Manifest;
use clap::*;
use execute::Execute;
use indicatif::MultiProgress;
use swift_bridge_build::{create_package, parse_bridges, ApplePlatform, CreatePackageConfig};

use crate::{CommandInfo, CommandSpinner, MainSpinner, Target, TargetInfo};

pub fn run(platforms: Option<Vec<Platform>>, package_name: Option<String>) {
    // TODO: Allow path as optional argument to take other directories than current directory
    let manifest =
        Manifest::from_path("./Cargo.toml").expect("Could not find Cargo.toml in this directory!");

    let platforms = platforms.unwrap_or_else(|| todo!("TODO: Interactive prompt!"));
    // TODO: Prompt this but suggest default name based on crate name
    let package_name = package_name.unwrap_or_else(|| todo!("Prompt!"));
    let crate_name = manifest.package.unwrap().name.to_lowercase();

    if platforms.is_empty() {
        eprintln!("At least 1 platform needs to be selected!");
        return;
    }

    generate_bridge_with_output(&crate_name);

    let targets: Vec<_> = platforms
        .into_iter()
        .flat_map(|p| p.into_apple_platforms())
        .map(|p| p.target())
        .collect();

    for target in &targets {
        build_with_output(target, &crate_name);
    }

    create_package_with_output(&targets, &crate_name, &package_name);
}

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value()]
pub enum Platform {
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

fn generate_bridge_with_output(crate_name: &str) {
    // TODO: Allow setting a base path here
    let out_dir = PathBuf::from("./generated");
    {
        let _gag = gag::Gag::stdout().unwrap();
        let parsed = parse_bridges(vec!["./src/lib.rs"]);
        parsed.write_all_concatenated(out_dir, crate_name);
    }
    let spinner = MainSpinner::with_message(format!("Generating Swift bridging header..."));
    spinner.finish();
}

fn build_with_output(target: &Target, crate_name: &str) {
    let multi = MultiProgress::new();
    let spinner = MainSpinner::with_target(target.clone());
    multi.add(spinner.clone().into());
    for mut command in target.commands(crate_name) {
        let step = CommandSpinner::with_command(&command);
        multi.add(step.clone().into());

        command
            .execute()
            .expect(format!("Failed to execute build command: {}", command.info()).as_str());

        step.finish()
    }
    spinner.finish();
}

fn create_package_with_output(targets: &[Target], crate_name: &str, package_name: &str) {
    // TODO: Use base path here
    let target_paths = targets
        .iter()
        .map(|t| (t.platform(), t.framework_path(crate_name).into()))
        .collect();
    let config = CreatePackageConfig {
        bridge_dir: PathBuf::from("./generated"),
        paths: target_paths,
        out_dir: package_name.into(),
        package_name: package_name.into(),
    };

    println!("");

    let spinner = MainSpinner::with_message(format!("Creating Swift Package '{package_name}'..."));
    create_package(config);
    spinner.finish();
}
