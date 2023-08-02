use std::fmt::Display;
use std::fs::{create_dir, write};
use std::process::Stdio;

use clap::ValueEnum;
use execute::{command, Execute};

use crate::config::Config;
use crate::error::Result;
use crate::lib_type::LibType;
use crate::step::run_step;

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
) -> Result<()> {
    run_step(&config, "Creating Rust library package...", || {
        create_project(&crate_name, lib_type, plain)
    })?;

    match vcs {
        Vcs::Git => init_git_repository(&crate_name, &config)?,
        Vcs::None => (),
    };

    Ok(())
}

fn create_project(crate_name: &str, lib_type: LibType, plain: bool) -> Result<()> {
    // let manifest = Manifest::from_str(include_str!("../../Cargo.toml")).unwrap();
    // let cargo_swift_version = manifest.package().version();

    let cargo_toml_content = include_str!("../../template/template.Cargo.toml")
        .replace("<CRATE_NAME>", crate_name)
        .replace("<LIB_TYPE>", lib_type.identifier());
    let (lib_rs_content, udl_content) = if plain {
        (
            include_str!("../../template/template.plain.rs"),
            include_str!("../../template/template.plain.udl"),
        )
    } else {
        (
            include_str!("../../template/template.lib.rs"),
            include_str!("../../template/template.lib.udl"),
        )
    };
    let build_rs_content = include_str!("../../template/template.build.rs");

    write_project_files(
        &cargo_toml_content,
        build_rs_content,
        lib_rs_content,
        udl_content,
        crate_name,
    )?;

    Ok(())
}

fn write_project_files(
    cargo_toml: &str,
    build_rs: &str,
    lib_rs: &str,
    lib_udl: &str,
    crate_name: &str,
) -> Result<()> {
    create_dir(crate_name).map_err(|_| "Could not create directory for crate!")?;

    write(format!("{}/Cargo.toml", crate_name), cargo_toml)
        .map_err(|_| "Could not write Cargo.toml!")?;
    write(format!("{}/build.rs", crate_name), build_rs).expect("Could not write build.rs!");

    create_dir(format!("{}/src", crate_name)).expect("Could not create src/ directory!");
    write(format!("{}/src/lib.rs", crate_name), lib_rs)
        .map_err(|_| "Could not write src/lib.rs!")?;
    write(format!("{}/src/lib.udl", crate_name), lib_udl)
        .map_err(|_| "Could not write src/lib.udl!")?;

    Ok(())
}

fn init_git_repository(crate_name: &str, config: &Config) -> Result<()> {
    let gitignore_content = include_str!("../../template/template.gitignore");
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
