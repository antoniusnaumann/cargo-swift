use std::fs::remove_dir_all;
use std::io;
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
use swift_bridge_build::CreatePackageConfig;

use crate::bindings::generate_bindings;
use crate::spinners::*;
use crate::targets::*;
use crate::xcframework::create_xcframework;
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

    generate_bindings_with_output(config.silent);

    for target in &targets {
        build_with_output(target, &crate_name, config.silent, mode);
    }

    recreate_output_dir(&package_name).expect("Could not create package output directory!");
    create_xcframework_with_output(&targets, &crate_name, &package_name, mode, config.silent);
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
            Platform::Ios => vec![ApplePlatform::IOS, ApplePlatform::IOSSimulator],
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
        .defaults(&[true, true, true, false])
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
        .split('\n')
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
    let multi = silent.not().then(MultiProgress::new);
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message("Installing Toolchains...".to_owned()));
    multi.add(&spinner);
    spinner.start();
    for toolchain in toolchains {
        let mut install = Command::new("rustup");
        install.args(["target", "install", toolchain]);
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

fn generate_bindings_with_output(silent: bool) {
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message(format!("Generating Swift bindings...")));

    generate_bindings().expect("Could not generate UniFFI bindings for udl files!");

    spinner.finish();
}

fn build_with_output(target: &Target, crate_name: &str, silent: bool, mode: Mode) {
    let multi = silent.not().then(MultiProgress::new);
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
            .unwrap_or_else(|_| panic!("Failed to execute build command: {}", command.info()));

        step.finish();
    }

    spinner.finish();
}

fn recreate_output_dir(package_name: &str) -> io::Result<()> {
    let dir = format!("./{package_name}");

    match remove_dir_all(dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

fn create_xcframework_with_output(
    targets: &[Target],
    crate_name: &str,
    package_name: &str,
    mode: Mode,
    silent: bool,
) {
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message(format!("Creating XCFramework...")));

    // TODO: show command spinner here with xcbuild command
    let output_dir = PathBuf::from(package_name);
    // TODO: make this configurable
    let generated_dir = PathBuf::from("./generated");
    create_xcframework(targets, crate_name, &generated_dir, &output_dir, mode).unwrap();

    spinner.finish();
}
