use std::ops::Not;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cargo_toml::Manifest;
use clap::ValueEnum;
use convert_case::{Case, Casing};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect};
use execute::{command, Execute};
use indicatif::MultiProgress;
use swift_bridge_build::{ApplePlatform, CreatePackageConfig};

use crate::*;

pub fn run(
    platforms: Option<Vec<Platform>>,
    package_name: Option<String>,
    config: Config,
    mode: Mode,
) {
    // TODO: Allow path as optional argument to take other directories than current directory
    let manifest =
        Manifest::from_path("./Cargo.toml").expect("Could not find Cargo.toml in this directory!");

    let crate_name = manifest.package.unwrap().name.to_lowercase();
    let package_name =
        package_name.unwrap_or_else(|| prompt_package_name(&crate_name, config.accept_all));
    let platforms = platforms.unwrap_or_else(|| prompt_platforms(config.accept_all));

    if platforms.is_empty() {
        eprintln!("At least 1 platform needs to be selected!");
        return;
    }

    let targets: Vec<_> = platforms
        .into_iter()
        .flat_map(|p| p.into_apple_platforms())
        .map(|p| p.target())
        .collect();

    let missing_toolchains = check_installed_toolchains(&targets);
    if !missing_toolchains.is_empty() {
        if config.accept_all || prompt_toolchain_installation(&missing_toolchains) {
            install_toolchains(&missing_toolchains, config.silent)
                .expect("Error while installing toolchains. Is rustup installed?");
        } else {
            eprintln!("Toolchains for some target platforms were missing!");
            return;
        }
    }

    generate_bridge_with_output(&crate_name, config.silent);

    for target in &targets {
        build_with_output(target, &crate_name, config.silent, mode);
    }

    create_package_with_output(&targets, &crate_name, &package_name, config.silent, mode);
}

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value()]
pub enum Platform {
    Macos,
    Ios,
    // Platforms below are removed until they are appropriately supported
    //    Tvos,
    //    Watchos,
}

impl Platform {
    fn into_apple_platforms(self) -> Vec<ApplePlatform> {
        match self {
            Platform::Macos => vec![ApplePlatform::MacOS],
            Platform::Ios => vec![ApplePlatform::IOS, ApplePlatform::Simulator],
            //            Platform::Tvos => vec![ApplePlatform::TvOS],
            //            Platform::Watchos => vec![ApplePlatform::WatchOS],
        }
    }

    fn display_name(&self) -> &'static str {
        match self {
            Platform::Macos => "macOS",
            Platform::Ios => "iOS",
            //            Platform::Tvos => "tvOS",
            //            Platform::Watchos => "watchOS",
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Macos,
            Self::Ios,
            //    Self::Tvos,
            //    Self::Watchos
        ]
    }
}

fn prompt_platforms(accept_all: bool) -> Vec<Platform> {
    let platforms = Platform::all();
    let items: Vec<_> = platforms.iter().map(|p| p.display_name()).collect();

    if accept_all {
        return platforms;
    }

    let chosen: Vec<usize> = MultiSelect::with_theme(&ColorfulTheme::default())
        .items(&items)
        .with_prompt("Select Target Platforms")
        .defaults(&vec![true, true, true, false])
        .interact()
        .unwrap();

    chosen.into_iter().map(|i| platforms[i]).collect()
}

/// Checks if toolchains for all target architectures are installed and returns a
/// list containing the names of all missing toolchains
fn check_installed_toolchains(targets: &[Target]) -> Vec<&'static str> {
    let mut rustup = command!("rustup target list");
    rustup.stdout(Stdio::piped());
    let output = rustup
        .execute_output()
        .expect("Failed to check installed toolchains. Is rustup installed on your system?");
    let output = String::from_utf8_lossy(&output.stdout);

    let installed: Vec<_> = output
        .split("\n")
        .filter(|s| s.contains("installed"))
        .map(|s| s.replace("(installed)", "").trim().to_owned())
        .collect();

    targets
        .iter()
        .flat_map(|t| t.architectures())
        .filter(|arch| {
            !installed
                .iter()
                .any(|toolchain| toolchain.eq_ignore_ascii_case(arch))
        })
        .collect()
}

/// Prompts the user to install the given **toolchains** by name
fn prompt_toolchain_installation(toolchains: &[&str]) -> bool {
    println!("The following toolchains are not installed:");

    for toolchain in toolchains {
        println!("\t{toolchain}")
    }

    let answer = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to install them? [Y/n]")
        .default("yes".to_owned())
        .interact_text()
        .unwrap()
        .trim()
        .to_lowercase();

    answer.eq_ignore_ascii_case("yes") || answer.eq_ignore_ascii_case("y")
}

/// Attempts to install the given **toolchains**
fn install_toolchains(toolchains: &[&str], silent: bool) -> Result<(), String> {
    let multi = silent.not().then(|| MultiProgress::new());
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message("Installing Toolchains...".to_owned()));
    multi.add(&spinner);
    spinner.start();
    for toolchain in toolchains {
        let mut install = Command::new("rustup");
        install.args(["target", "install", &toolchain]);
        install.stdin(Stdio::null());

        let step = silent.not().then(|| CommandSpinner::with_command(&install));
        multi.add(&step);
        step.start();

        install
            .execute()
            .map_err(|e| format!("Error while donwloading toolchain {toolchain}: \n\t{e}"))?;

        step.finish();
    }
    spinner.finish();

    Ok(())
}

fn prompt_package_name(crate_name: &str, accept_all: bool) -> String {
    let default = crate_name.to_case(Case::UpperCamel);

    if accept_all {
        return default;
    }

    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Swift Package Name")
        .default(default)
        .interact_text()
        .unwrap()
}

fn generate_bridge_with_output(crate_name: &str, silent: bool) {
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message(format!("Generating Swift bridging header...")));
    // TODO: Allow setting a base path here
    let out_dir = PathBuf::from("./generated");
    let parsed = swift_bridge_build::parse_bridges(vec!["./src/lib.rs"]);
    parsed.write_all_concatenated(out_dir, crate_name);

    spinner.finish();
}

fn build_with_output(target: &Target, crate_name: &str, silent: bool, mode: Mode) {
    let multi = silent.not().then(|| MultiProgress::new());
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_target(target.clone()));
    multi.add(&spinner);
    spinner.start();

    for mut command in target.commands(crate_name, mode) {
        let step = silent.not().then(|| CommandSpinner::with_command(&command));
        multi.add(&step);
        step.start();

        command
            .execute()
            .expect(format!("Failed to execute build command: {}", command.info()).as_str());

        step.finish();
    }

    spinner.finish();
}

fn create_package_with_output(
    targets: &[Target],
    crate_name: &str,
    package_name: &str,
    silent: bool,
    mode: Mode,
) {
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message(format!("Creating Swift Package '{package_name}'...")));
    // TODO: Use base path here
    let target_paths = targets
        .iter()
        .map(|t| (t.platform(), t.framework_path(crate_name, mode).into()))
        .collect();
    let config = CreatePackageConfig {
        bridge_dir: PathBuf::from("./generated"),
        paths: target_paths,
        out_dir: package_name.into(),
        package_name: package_name.into(),
    };

    swift_bridge_build::create_package(config);

    spinner.finish();

    let spinner = silent.not().then(|| {
        MainSpinner::with_message(format!(
            "Successfully created Swift Package in '{package_name}/'!"
        ))
    });
    spinner.finish();
}
