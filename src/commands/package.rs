use cargo_toml::Manifest;
use clap::*;
use execute::Execute;
use indicatif::MultiProgress;
use swift_bridge_build::ApplePlatform;

use crate::{CommandInfo, CommandSpinner, Target, TargetInfo, TargetSpinner};

pub fn run(platforms: Option<Vec<Platform>>, package_name: Option<String>) {
    println!("Package");
    // TODO: Allow path as optional argument to take other directories than current directory
    let manifest =
        Manifest::from_path("./Cargo.toml").expect("Could not find Cargo.toml in this directory!");

    let platforms = platforms.unwrap_or_else(|| todo!("TODO: Interactive prompt!"));
    // TODO: Prompt this but suggest default name based on crate name
    let package_name = package_name.unwrap_or_else(|| todo!("Prompt!"));
    let crate_name = dbg!(manifest.package.unwrap().name.to_lowercase());

    if platforms.is_empty() {
        eprintln!("At least 1 platform needs to be selected!");
        return;
    }

    platforms
        .into_iter()
        .flat_map(|p| p.into_apple_platforms())
        .map(|p| p.target())
        .for_each(|target| build_with_output(target, &crate_name));
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

fn build_with_output(target: Target, crate_name: &str) {
    let multi = MultiProgress::new();
    let spinner = TargetSpinner::with_target(target.clone());
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
