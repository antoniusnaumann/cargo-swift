use std::fs::{create_dir, write};
use std::ops::Not;
use std::process::Stdio;

use cargo_toml::Manifest;
use execute::{command, Execute};

use crate::Ticking;
use crate::{config::Config, MainSpinner};

pub fn run(crate_name: String, config: Config) {
    let spinner = config.silent.not().then_some(MainSpinner::with_message(
        "Creating Rust library package...".to_owned(),
    ));
    spinner.start();

    create_dir(&crate_name).expect("Could not create directory for crate!");

    let manifest = Manifest::from_str(include_str!("../../Cargo.toml")).unwrap();
    let cargo_swift_version = manifest.package().version();

    let gitignore_content = include_str!("../../template/template.gitignore");
    let cargo_toml_content =
        include_str!("../../template/template.toml").replace("<CRATE_NAME>", &crate_name);
    let lib_rs_content =
        include_str!("../../template/lib.rs").replace("<CARGO_SWIFT_VERSION>", cargo_swift_version);

    write(format!("{}/.gitignore", crate_name), gitignore_content)
        .expect("Could not write .gitignore!");

    write(format!("{}/Cargo.toml", crate_name), cargo_toml_content)
        .expect("Could not write Cargo.toml!");

    create_dir(format!("{}/src", crate_name)).expect("Could not create src/ directory!");
    write(format!("{}/src/lib.rs", crate_name), lib_rs_content)
        .expect("Could not write src/lib.rs!");

    spinner.finish();

    let git_status_output = command!("git status")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .execute_output()
        .expect("Could not run git status!");

    if let Some(0) = git_status_output.status.code() {
        // Already in a git repository
        return;
    }

    let spinner = config.silent.not().then_some(MainSpinner::with_message(
        "Initializing git repository...".to_owned(),
    ));
    spinner.start();

    command!("git init")
        .current_dir(format!("./{crate_name}"))
        .execute_check_exit_status_code(0)
        .expect("Could not initialize git repository!");
    command!("git checkout -b main")
        .current_dir(format!("./{crate_name}"))
        .execute_check_exit_status_code(0)
        .expect("Could not checkout branch main!");

    spinner.finish();
}
