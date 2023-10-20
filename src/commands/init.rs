use askama::Template;
use std::fmt::Display;
use std::fs::{create_dir, write};
use std::process::Stdio;

use clap::ValueEnum;
use execute::{command, Execute};

use crate::console::{run_step, Config, Result};
use crate::lib_type::LibType;
use crate::templating;

#[derive(ValueEnum, Debug, Clone)]
#[value()]
pub enum Vcs {
    Git,
    None,
}

impl Display for Vcs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git => write!(f, "git"),
            Self::None => write!(f, "none"),
        }
    }
}

pub fn run(
    crate_name: String,
    config: Config,
    vcs: Vcs,
    lib_type: LibType,
    plain: bool,
    macro_only: bool,
) -> Result<()> {
    run_step(&config, "Creating Rust library package...", || {
        create_project(&crate_name, lib_type, plain, macro_only)
    })?;

    match vcs {
        Vcs::Git => init_git_repository(&crate_name, &config)?,
        Vcs::None => (),
    };

    Ok(())
}

fn create_project(
    crate_name: &str,
    lib_type: LibType,
    plain: bool,
    macro_only: bool,
) -> Result<()> {
    // let manifest = Manifest::from_str(include_str!("../../Cargo.toml")).unwrap();
    // let cargo_swift_version = manifest.package().version();
    let namespace = crate_name.replace('-', "_");
    let cargo_toml_content = templating::CargoToml {
        crate_name,
        namespace: &namespace,
        lib_type: lib_type.identifier(),
        macro_only,
    };
    let lib_rs_content = templating::LibRs { plain, macro_only };
    let (udl_content, build_rs_content) = if !macro_only {
        (
            Some(templating::LibUdl {
                namespace: &namespace,
                plain,
            }),
            Some(templating::BuildRs {}),
        )
    } else {
        (None, None)
    };

    write_project_files(
        cargo_toml_content,
        build_rs_content,
        lib_rs_content,
        udl_content,
        crate_name,
    )?;

    Ok(())
}

fn write_project_files(
    cargo_toml: templating::CargoToml,
    build_rs: Option<templating::BuildRs>,
    lib_rs: templating::LibRs,
    lib_udl: Option<templating::LibUdl>,
    crate_name: &str,
) -> Result<()> {
    create_dir(crate_name).map_err(|_| "Could not create directory for crate!")?;

    write(
        format!("{}/Cargo.toml", crate_name),
        cargo_toml.render().unwrap(),
    )
    .map_err(|_| "Could not write Cargo.toml!")?;

    if let Some(build_rs) = build_rs {
        write(
            format!("{}/build.rs", crate_name),
            build_rs.render().unwrap(),
        )
        .expect("Could not write build.rs!");
    }

    create_dir(format!("{}/src", crate_name)).expect("Could not create src/ directory!");
    write(
        format!("{}/src/lib.rs", crate_name),
        lib_rs.render().unwrap(),
    )
    .map_err(|_| "Could not write src/lib.rs!")?;

    if let Some(lib_udl) = lib_udl {
        write(
            format!("{}/src/lib.udl", crate_name),
            lib_udl.render().unwrap(),
        )
        .map_err(|_| "Could not write src/lib.udl!")?;
    }

    Ok(())
}

fn init_git_repository(crate_name: &str, config: &Config) -> Result<()> {
    let gitignore_content = include_str!("../../templates/template.gitignore");
    write(format!("{}/.gitignore", crate_name), gitignore_content)
        .map_err(|_| "Could not write .gitignore!")?;

    let git_status_output = command!("git status")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .execute_output()
        .expect("Could not run git status!");

    if let Some(0) = git_status_output.status.code() {
        // Already in a git repository
        return Ok(());
    }

    run_step(config, "Initializing git repository...", || {
        create_git_repo(crate_name)
    })?;

    Ok(())
}

fn create_git_repo(crate_name: &str) -> Result<()> {
    command!("git init")
        .current_dir(format!("./{crate_name}"))
        .execute_check_exit_status_code(0)
        .map_err(|_| "Could not initialize git repository!")?;
    command!("git checkout -b main")
        .current_dir(format!("./{crate_name}"))
        .execute_check_exit_status_code(0)
        .map_err(|_| "Could not checkout branch main!")?;

    Ok(())
}
