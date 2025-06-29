use std::fmt::Display;
use std::ops::Not;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use camino::Utf8PathBuf;
use cargo_metadata::{Package, TargetKind};
use clap::ValueEnum;
use convert_case::{Case, Casing};
use dialoguer::{Input, MultiSelect};
use execute::{command, Execute};
use indicatif::MultiProgress;

use crate::bindings::generate_bindings;
use crate::console::*;
use crate::console::{run_step, run_step_with_commands};
use crate::lib_type::LibType;
use crate::metadata::{metadata, MetadataExt};
use crate::swiftpackage::{create_swiftpackage, recreate_output_dir};
use crate::targets::*;
use crate::xcframework::create_xcframework;

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

#[derive(Debug, Clone)]
pub struct FeatureOptions {
    pub features: Option<Vec<String>>,
    pub all_features: bool,
    pub no_default_features: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    platforms: Option<Vec<Platform>>,
    build_target: Option<&str>,
    package_name: Option<String>,
    xcframework_name: String,
    disable_warnings: bool,
    config: Config,
    mode: Mode,
    lib_type_arg: LibTypeArg,
    features: FeatureOptions,
    skip_toolchains_check: bool,
) -> Result<()> {
    // TODO: Allow path as optional argument to take other directories than current directory
    // let crates = metadata().uniffi_crates();
    let crates = [metadata()
        .current_crate()
        .ok_or("Current directory is not part of a crate!")?];

    if crates.len() == 1 {
        return run_for_crate(
            crates[0],
            platforms.clone(),
            build_target,
            package_name,
            xcframework_name,
            disable_warnings,
            &config,
            mode,
            lib_type_arg,
            features,
            skip_toolchains_check,
        );
    } else if package_name.is_some() {
        Err("Package name can only be specified when building a single crate!")?;
    }

    crates
        .iter()
        .map(|current_crate| {
            info!(&config, "Packaging crate {}", current_crate.name);
            run_for_crate(
                current_crate,
                platforms.clone(),
                build_target,
                None,
                xcframework_name.clone(),
                disable_warnings,
                &config,
                mode,
                lib_type_arg.clone(),
                features.clone(),
                skip_toolchains_check,
            )
        })
        .filter_map(|result| result.err())
        .collect::<Errors>()
        .into()
}

#[allow(clippy::too_many_arguments)]
fn run_for_crate(
    current_crate: &Package,
    platforms: Option<Vec<Platform>>,
    build_target: Option<&str>,
    package_name: Option<String>,
    xcframework_name: String,
    disable_warnings: bool,
    config: &Config,
    mode: Mode,
    lib_type_arg: LibTypeArg,
    features: FeatureOptions,
    skip_toolchains_check: bool,
) -> Result<()> {
    let lib = current_crate
        .targets
        .iter()
        .find(|t| t.kind.contains(&TargetKind::Lib))
        .ok_or("No library tag defined in Cargo.toml!")?;
    let lib_types = lib
        .crate_types
        .iter()
        .filter_map(|t| t.clone().try_into().ok())
        .collect::<Vec<_>>();
    let lib_type = pick_lib_type(&lib_types, lib_type_arg.clone().into(), config)?;

    if lib_type == LibType::Dynamic {
        warning!(
            &config,
            "Building as dynamic library is discouraged. It might prevent apps that use this library from publishing to the App Store."
        );
    }

    let crate_name = current_crate.name.to_lowercase();
    let package_name =
        package_name.unwrap_or_else(|| prompt_package_name(&crate_name, config.accept_all));

    let platforms = platforms.unwrap_or_else(|| prompt_platforms(config.accept_all));

    if platforms.is_empty() {
        Err("At least 1 platform needs to be selected!")?;
    }

    let mut targets: Vec<_> = platforms
        .into_iter()
        .flat_map(|p| p.into_apple_platforms())
        .map(|p| p.target())
        .collect();

    if let Some(build_target) = build_target {
        targets.retain_mut(|platform_target| match platform_target {
            Target::Single { architecture, .. } => *architecture == build_target,
            Target::Universal {
                architectures,
                display_name,
                platform,
                ..
            } => {
                let Some(architecture) = architectures.iter().find(|t| **t == build_target) else {
                    return false;
                };
                *platform_target = Target::Single {
                    architecture,
                    display_name,
                    platform: *platform,
                };
                true
            }
        });
        if targets.is_empty() {
            return Err(Error::from(format!(
                "No matching build target for {}",
                build_target
            )));
        }
    }

    if !skip_toolchains_check {
        let missing_toolchains = check_installed_toolchains(&targets);
        let nightly_toolchains = check_nightly_installed(&targets);

        let installation_required =
            &[missing_toolchains.as_slice(), nightly_toolchains.as_slice()].concat();

        if !installation_required.is_empty() {
            if config.accept_all || prompt_toolchain_installation(installation_required) {
                install_toolchains(&missing_toolchains, config.silent)?;
                if !nightly_toolchains.is_empty() {
                    install_nightly_src(config.silent)?;
                }
            } else {
                Err("Toolchains for some target platforms were missing!")?;
            }
        }
    }

    let crate_name = lib.name.replace('-', "_");
    for target in &targets {
        build_with_output(target, &crate_name, mode, lib_type, config, &features)?;
    }

    generate_bindings_with_output(&targets, &crate_name, mode, lib_type, config)?;

    recreate_output_dir(&package_name).expect("Could not create package output directory!");
    create_xcframework_with_output(
        &targets,
        &crate_name,
        &package_name,
        &xcframework_name,
        mode,
        lib_type,
        config,
    )?;
    create_package_with_output(&package_name, &xcframework_name, disable_warnings, config)?;

    Ok(())
}

// FIXME: This can be removed once variant_count is stabilized: https://doc.rust-lang.org/std/mem/fn.variant_count.html#:~:text=Function%20std%3A%3Amem%3A%3Avariant_count&text=Returns%20the%20number%20of%20variants,the%20return%20value%20is%20unspecified.
const PLATFORM_COUNT: usize = 5;

#[derive(ValueEnum, Copy, Clone, Debug)]
#[value()]
pub enum Platform {
    Macos,
    Ios,
    // Platforms below are experimental
    Tvos,
    Watchos,
    Visionos,
    Maccatalyst,
}

impl Platform {
    fn into_apple_platforms(self) -> Vec<ApplePlatform> {
        match self {
            Platform::Macos => vec![ApplePlatform::MacOS],
            Platform::Ios => vec![ApplePlatform::IOS, ApplePlatform::IOSSimulator],
            Platform::Tvos => vec![ApplePlatform::TvOS, ApplePlatform::TvOSSimulator],
            Platform::Watchos => vec![ApplePlatform::WatchOS, ApplePlatform::WatchOSSimulator],
            Platform::Visionos => vec![ApplePlatform::VisionOS, ApplePlatform::VisionOSSimulator],
            Platform::Maccatalyst => vec![ApplePlatform::MacCatalyst],
        }
    }

    fn display_name(&self) -> String {
        let name = match self {
            Platform::Macos => "macOS",
            Platform::Ios => "iOS",
            Platform::Tvos => "tvOS",
            Platform::Watchos => "watchOS",
            Platform::Visionos => "visionOS",
            Platform::Maccatalyst => "Mac Catalyst",
        };

        format!(
            "{name}{}",
            if self.is_experimental() {
                " (Experimental)"
            } else {
                ""
            }
        )
    }

    fn is_experimental(&self) -> bool {
        match self {
            Platform::Macos | Platform::Ios => false,
            Platform::Tvos | Platform::Watchos | Platform::Visionos | Platform::Maccatalyst => true,
        }
    }

    fn all() -> [Self; PLATFORM_COUNT] {
        [
            Self::Macos,
            Self::Ios,
            Self::Tvos,
            Self::Watchos,
            Self::Visionos,
        ]
    }
}

fn prompt_platforms(accept_all: bool) -> Vec<Platform> {
    let platforms = Platform::all();
    let items = platforms.map(|p| p.display_name());

    if accept_all {
        return platforms
            .into_iter()
            .filter(|p| !p.is_experimental())
            .collect();
    }

    let theme = prompt_theme();
    let selector = MultiSelect::with_theme(&theme)
        .items(&items)
        .with_prompt("Select Target Platforms")
        // TODO: Move this to separate class and disable reporting to change style on success
        // .report(false)
        .defaults(&platforms.map(|p| !p.is_experimental()));

    let chosen: Vec<usize> = selector.interact().unwrap();

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
        .filter(|t| !t.platform().is_tier_3())
        .flat_map(|t| t.architectures())
        .filter(|arch| {
            !installed
                .iter()
                .any(|toolchain| toolchain.eq_ignore_ascii_case(arch))
        })
        .collect()
}

/// Checks if rust-src component for tier 3 targets are installed
fn check_nightly_installed(targets: &[Target]) -> Vec<&'static str> {
    if !targets.iter().any(|t| t.platform().is_tier_3()) {
        return vec![];
    }

    // TODO: Check if the correct nightly toolchain itself is installed
    let mut rustup = command("rustup component list --toolchain nightly");
    rustup.stdout(Stdio::piped());
    // HACK: Silence error that toolchain is not installed
    rustup.stderr(Stdio::null());

    let output = rustup
        .execute_output()
        .expect("Failed to check installed components. Is rustup installed on your system?");
    let output = String::from_utf8_lossy(&output.stdout);

    if output
        .split('\n')
        .filter(|s| s.contains("installed"))
        .map(|s| s.replace("(installed)", "").trim().to_owned())
        .any(|s| s.eq_ignore_ascii_case("rust-src"))
    {
        vec![]
    } else {
        vec!["rust-src (nightly)"]
    }
}

/// Prompts the user to install the given **toolchains** by name
fn prompt_toolchain_installation(toolchains: &[&str]) -> bool {
    println!("The following toolchains are not installed:");

    for toolchain in toolchains {
        println!("\t{toolchain}")
    }

    let theme = prompt_theme();
    let answer = Input::with_theme(&theme)
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
    if toolchains.is_empty() {
        return Ok(());
    };

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
            .map_err(|e| format!("Error while downloading toolchain {toolchain}: \n\t{e}"))?;

        step.finish();
    }
    spinner.finish();

    Ok(())
}

/// Attempts to install the "rust-src" component on nightly
fn install_nightly_src(silent: bool) -> Result<()> {
    let multi = silent.not().then(MultiProgress::new);
    let spinner = silent
        .not()
        .then(|| MainSpinner::with_message("Installing Toolchains...".to_owned()));
    multi.add(&spinner);
    spinner.start();

    let mut install = command("rustup toolchain install nightly");
    install.stdin(Stdio::null());

    let step = silent.not().then(|| CommandSpinner::with_command(&install));
    multi.add(&step);
    step.start();

    // TODO: make this a separate function and show error spinner on fail
    install
        .execute()
        .map_err(|e| format!("Error while installing rust-src on nightly: \n\t{e}"))?;

    step.finish();

    let mut install = command("rustup component add rust-src --toolchain nightly");
    install.stdin(Stdio::null());

    let step = silent.not().then(|| CommandSpinner::with_command(&install));
    multi.add(&step);
    step.start();

    // TODO: make this a separate function and show error spinner on fail
    install
        .execute()
        .map_err(|e| format!("Error while installing rust-src on nightly: \n\t{e}"))?;

    step.finish();
    spinner.finish();

    Ok(())
}

fn prompt_package_name(crate_name: &str, accept_all: bool) -> String {
    let default = crate_name.to_case(Case::UpperCamel);

    if accept_all {
        return default;
    }

    let theme = prompt_theme();
    Input::with_theme(&theme)
        .with_prompt("Swift Package Name")
        .default(default)
        .interact_text()
        .unwrap()
}

fn pick_lib_type(
    options: &[LibType],
    suggested: Option<LibType>,
    config: &Config,
) -> Result<LibType> {
    if let Some(result) = suggested.and_then(|t| options.iter().find(|&&i| t == i)) {
        return Ok(*result);
    }

    // TODO: Prompt user if multiple library types are present
    let choosen = if options.iter().any(|i| matches!(&i, LibType::Static)) {
        LibType::Static
    } else {
        *options.first().ok_or("No supported library type was specified in Cargo.toml! \n\n Supported types are: \n\t- staticlib \n\t- cdylib")?
    };

    if let Some(suggested) = suggested {
        // TODO: Show part of Cargo.toml here to help user fix this
        warning!(config,
            "No matching library type for --lib-type {suggested} found in Cargo.toml.\n  Building as {choosen} instead...")
    }
    Ok(choosen)
}

fn generate_bindings_with_output(
    targets: &[Target],
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
) -> Result<()> {
    run_step(config, "Generating Swift bindings...", || {
        let lib_file = library_file_name(lib_name, lib_type);
        let target = metadata().target_dir();
        let archs = targets
            .first()
            .ok_or("Could not generate UniFFI bindings: No target platform selected!")?
            .architectures();
        let arch = archs.first();
        let lib_path: Utf8PathBuf = format!("{target}/{arch}/{mode}/{lib_file}").into();

        generate_bindings(&lib_path)
            .map_err(|e| format!("Could not generate UniFFI bindings for udl files due to the following error: \n {e}").into())
    })
}

fn build_with_output(
    target: &Target,
    lib_name: &str,
    mode: Mode,
    lib_type: LibType,
    config: &Config,
    features: &FeatureOptions,
) -> Result<()> {
    let mut commands = target.commands(lib_name, mode, lib_type, features);
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
    xcframework_name: &str,
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
            xcframework_name,
            &generated_dir,
            &output_dir,
            mode,
            lib_type,
        )
    })
    .map_err(|e| format!("Failed to create XCFramework due to the following error: \n {e}").into())
}

fn create_package_with_output(
    package_name: &str,
    xcframework_name: &str,
    disable_warnings: bool,
    config: &Config,
) -> Result<()> {
    run_step(
        config,
        format!("Creating Swift Package '{package_name}'..."),
        || create_swiftpackage(package_name, xcframework_name, disable_warnings),
    )?;

    let spinner = config.silent.not().then(|| {
        MainSpinner::with_message(format!(
            "Successfully created Swift Package in '{package_name}/'!"
        ))
    });
    spinner.finish();

    Ok(())
}
