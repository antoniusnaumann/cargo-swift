use std::io::{stderr, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Result};

use crate::{Mode, Target};

pub fn create_xcframework(
    targets: &[Target],
    lib_name: &str,
    generated_dir: &Path,
    output_dir: &Path,
    mode: Mode,
) -> Result<()> {
    let libs: Vec<_> = targets
        .iter()
        .map(|t| t.library_file(lib_name, mode))
        .collect();

    let headers = generated_dir.join("headers");
    let headers = headers
        .to_str()
        .ok_or(anyhow!("Directory for bindings has an invalid name!"))?;

    let output_dir_name = &output_dir
        .to_str()
        .ok_or(anyhow!("Output directory has an invalid name!"))?;

    // TODO: this should be given as an argument instead of being hardcoded
    //  because it needs to match the name given in swift package manifest
    let framework = format!("{output_dir_name}/RustFramework.xcframework");

    let mut xcodebuild = Command::new("xcodebuild");
    xcodebuild.arg("-create-xcframework");

    for lib in &libs {
        xcodebuild.arg("-library");
        xcodebuild.arg(lib);
        xcodebuild.arg("-headers");
        xcodebuild.arg(headers);
    }

    let output = xcodebuild
        .arg("-output")
        .arg(&framework)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        // TODO: This method should not be responsible for printing error directly.
        //  Create a custom error enum that can hold either a string or a Vec<u8> instead.
        stderr().write_all(&output.stderr).unwrap();

        Err(anyhow!("xcodebuild failed!"))
    } else {
        Ok(())
    }
}
