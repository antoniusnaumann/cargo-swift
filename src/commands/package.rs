use std::fmt::Display;
use std::fs::read;
use std::ops::Not;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use camino::Utf8PathBuf;
use cargo_toml::Manifest;
use clap::ValueEnum;
use convert_case::{Case, Casing};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect};
use execute::{command, Execute};
use indicatif::MultiProgress;

use crate::bindings::generate_bindings;
use crate::error::*;
use crate::lib_type::LibType;
use crate::spinners::*;
use crate::step::{run_step, run_step_with_commands};
use crate::swiftpackage::{create_swiftpackage, recreate_output_dir};
use crate::targets::*;
use crate::xcframework::create_xcframework;
use crate::*;

#[derive(ValueEnum, Debug, Clone)]
#[value()]
pub enum LibTypeArg {
    Automatic,
    Dynamic,
    Static,
}

impl Display for LibTypeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Automatic => write!(f, "automatic"),
            Self::Static => write!(f, "static"),
            Self::Dynamic => write!(f, "dynamic"),
        }
    }
}

impl From<LibTypeArg> for Option<LibType> {
    fn from(value: LibTypeArg) -> Self {
        match value {
            LibTypeArg::Automatic => None,
            LibTypeArg::Dynamic => Some(LibType::Dynamic),
            LibTypeArg::Static => Some(LibType::Static),
        }
    }
}

pub fn run(
    platforms: Option<Vec<Platform>>,
    package_name: Option<String>,
    config: Config,
    mode: Mode,
    lib_type_arg: LibTypeArg,
) -> Result<()> {
    // TODO: Allow path as optional argument to take other directories than current directory
    let manifest = Manifest::from_slice(&read("./Cargo.toml")?)
        .expect("Could not find Cargo.toml in this directory!");

    let crate_name = manifest.package.unwrap().name.to_lowercase();
    let package_name =
        package_name.unwrap_or_else(|| prompt_package_name(&crate_name, config.accept_all));
    let lib = manifest
        .lib
        .ok_or("No library tag defined in Cargo.toml!")?;
    let lib_name = lib.name.ok_or("No library name found in Cargo.toml!")?;
    let lib_types = lib
        .crate_type
        .iter()
        .filter_map(|t| t.parse().ok())
        .collect::<Vec<_>>();
    let lib_type = pick_lib_type(&lib_types, lib_type_arg.into())?;

    let platforms = platforms.unwrap_or_else(|| prompt_platforms(config.accept_all));

    if platforms.is_empty() {
        Err("At least 1 platform needs to be selected!")?;
    }

    let targets: Vec<_> = platforms
        .into_iter()
        .flat_map(|p| p.into_apple_platforms())
        .map(|p| p.target())
        .collect();

    let missing_toolchains = check_installed_toolchains(&targets);
    if !missing_toolchains.is_empty() {
        if config.accept_all || prompt_toolchain_installation(&missing_toolchains) {
            install_toolchains(&missing_toolchains, config.silent)?;
        } else {
            Err("Toolchains for some target platforms were missing!")?;
        }
    }

    for target in &targets {
        build_with_output(target, &lib_name, mode, lib_type, &config)?;
    }

    let namespace = generate_bindings_with_output(&targets, &lib_name, mode, lib_type, &config)?;

    recreate_output_dir(&package_name).expect("Could not create package output directory!");
    create_xcframework_with_output(&targets, &lib_name, &package_name, mode, lib_type, &config)?;
    create_package_with_output(&package_name, &namespace, &config)?;

    Ok(())
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
fn install_toolchains(toolchains: &[&str], silent: bool) -> Result<()> {
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

        // TODO: make this a separate function and show error spinner on fail
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

fn pick_lib_type(options: &[LibType], suggested: Option<LibType>) -> Result<LibType> {
    // TODO: ERROR HANDLING if a non-matching type is given, this should return an error instead of defaulting to automatic
    if let Some(result) = suggested.and_then(|t| options.iter().find(|&&i| t == i)) {
        return Ok(*result);
    }

    // TODO: Remove on next breaking version bump, this is only here to not induce a breaking change in the tools behavior
    if options.iter().any(|i| matches!(&i, LibType::Dynamic)) {
        return Ok(LibType::Dynamic);
    }

    // TODO: Prompt user if multiple library types are present
    let first = options.first().ok_or("No supported library type was specified in Cargo.toml! \n\n Supported types are: \n\t- staticlib \n\t- cdylib")?;

    Ok(*first)
}

fn generate_bindings_with_output(
    targets: &[Target],
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<String> {
    run_step(config, "Generating Swift bindings...", || {
        let lib_file = library_file_name(lib_name, lib_type);
        let target = target_dir();
        let lib_path: Option<Utf8PathBuf> = targets
            .first()
            .and_then(|t| t.architectures().first().cloned())
            .map(|arch| format!("{target}/{arch}/{mode}/{lib_file}").into());
        generate_bindings(lib_path.as_deref())
            .map_err(|e| format!("Could not generate UniFFI bindings for udl files due to the following error: \n {e}").into())
    })
}

fn build_with_output(
    target: &Target,
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<()> {
    let mut commands = target.commands(lib_name, mode, lib_type);
    for command in &mut commands {
        command.env("CARGO_TERM_COLOR", "always");
    }

    run_step_with_commands(
        config,
        format!("Building target {}", target.display_name()),
        &mut commands,
    )?;

    Ok(())
}

fn create_xcframework_with_output(
    targets: &[Target],
    lib_name: &str,
    package_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<()> {
    run_step(config, "Creating XCFramework...", || {
        // TODO: show command spinner here with xcbuild command
        let output_dir = PathBuf::from(package_name);
        // TODO: make this configurable
        let generated_dir = PathBuf::from("./generated");

        create_xcframework(
            targets,
            lib_name,
            &generated_dir,
            &output_dir,
            mode,
            lib_type,
        )
    })
    .map_err(|e| format!("Failed to create XCFramework due to the following error: \n {e}").into())
}

fn create_package_with_output(package_name: &str, namespace: &str, config: &Config) -> Result<()> {
    run_step(
        config,
        format!("Creating Swift Package '{package_name}'..."),
        || create_swiftpackage(package_name, namespace),
    )?;

    let spinner = config.silent.not().then(|| {
        MainSpinner::with_message(format!(
            "Successfully created Swift Package in '{package_name}/'!"
        ))
    });
    spinner.finish();

    Ok(())
}
