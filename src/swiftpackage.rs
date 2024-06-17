use askama::Template;
use glob::glob;
use std::fs::{copy, create_dir_all, write};

use crate::{recreate_dir, templating, Result};

/// Create artifacts for a swift package given the package name
///
/// **Note**: This method assumes that a directory with the package name and the .xcframework already exists
pub fn create_swiftpackage(
    package_name: &str,
    xcframework_name: &str,
    disable_warnings: bool,
) -> Result<()> {
    // TODO: Instead of assuming the directory and the xcframework, let this manage directory
    //  recreation and let it copy the xcframework
    let package_manifest = templating::PackageSwift {
        package_name,
        xcframework_name,
        disable_warnings,
    };

    write(
        format!("{}/Package.swift", package_name),
        package_manifest.render().unwrap(),
    )
    .map_err(|e| format!("Could not write Package.swift: \n {e}"))?;

    create_dir_all(format!("{}/Sources/{}", package_name, package_name))
        .map_err(|e| format!("Could not create module sources directory: \n {e}"))?;

    for swift_file in glob("./generated/sources/*.swift")
        .map_err(|e| format!("Could not find generated swift source files: \n {e}"))?
    {
        let swift_file = swift_file
            .map_err(|e| format!("Could not find generated swift source file: \n {e}"))?;
        let file_name = swift_file
            .file_name()
            .ok_or("Could not get file name")?
            .to_str()
            .ok_or("Could not convert file name to string")?
            .to_string();
        copy(
            swift_file,
            format!("{}/Sources/{}/{}", package_name, package_name, file_name),
        )
        .map_err(|e| format!("Could not copy generated swift source files: \n {e}"))?;
    }

    Ok(())
}

pub fn recreate_output_dir(package_name: &str) -> Result<()> {
    let dir = format!("./{package_name}");

    recreate_dir(dir)
}
