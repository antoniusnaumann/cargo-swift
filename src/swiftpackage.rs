use std::fs::{copy, create_dir_all, write};

use crate::{recreate_dir, Result};

/// Create artifacts for a swift package given the package name
///
/// **Note**: This method assumes that a directory with the package name and the .xcframework already exists
pub fn create_swiftpackage(package_name: &str) -> Result<()> {
    // TODO: Instead of assuming the directory and the xcframework, let this manage directory
    //  recreation and let it copy the xcframework
    let package_manifest =
        include_str!("../template/template.Package.swift").replace("<PACKAGE_NAME>", package_name);

    write(format!("{}/Package.swift", package_name), package_manifest)
        .map_err(|e| format!("Could not write Package.swift: \n {e}"))?;

    create_dir_all(format!("{}/Sources/{}", package_name, package_name))
        .map_err(|e| format!("Could not create module sources directory: \n {e}"))?;

    copy(
        "./generated/sources/lib.swift",
        format!("{}/Sources/{}/lib.swift", package_name, package_name),
    )
    .map_err(|e| format!("Could not copy generated swift source files: \n {e}"))?;

    Ok(())
}

pub fn recreate_output_dir(package_name: &str) -> Result<()> {
    let dir = format!("./{package_name}");

    recreate_dir(dir)
}
