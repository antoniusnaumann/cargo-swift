use askama::Template;
use std::fs::{copy, create_dir_all, write};

use crate::{recreate_dir, templating, Result};

/// Create artifacts for a swift package given the package name
///
/// **Note**: This method assumes that a directory with the package name and the .xcframework already exists
pub fn create_swiftpackage(package_name: &str, namespace: &str, disable_warnings: bool) -> Result<()> {
    // TODO: Instead of assuming the directory and the xcframework, let this manage directory
    //  recreation and let it copy the xcframework
    let package_manifest = templating::PackageSwift {
        package_name,
        disable_warnings,
    };

    write(
        format!("{}/Package.swift", package_name),
        package_manifest.render().unwrap(),
    )
    .map_err(|e| format!("Could not write Package.swift: \n {e}"))?;

    create_dir_all(format!("{}/Sources/{}", package_name, package_name))
        .map_err(|e| format!("Could not create module sources directory: \n {e}"))?;

    copy(
        format!("./generated/sources/{namespace}.swift"),
        format!(
            "{}/Sources/{}/{}.swift",
            package_name, package_name, namespace
        ),
    )
    .map_err(|e| format!("Could not copy generated swift source files: \n {e}"))?;

    Ok(())
}

pub fn recreate_output_dir(package_name: &str) -> Result<()> {
    let dir = format!("./{package_name}");

    recreate_dir(dir)
}
