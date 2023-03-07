use std::fs::{copy, create_dir_all, write};
use std::io;

use crate::recreate_dir;

/// Create artifacts for a swift package given the package name
///
/// **Note**: This method assumes that a directory with the package name and the .xcframework already exists
pub fn create_swiftpackage(package_name: &str) {
    // TODO: Instead of assuming the directory and the xcframework, let this manage directory
    //  recreation and let it copy the xcframework
    let package_manifest =
        include_str!("../template/template.Package.swift").replace("<PACKAGE_NAME>", package_name);

    write(format!("{}/Package.swift", package_name), package_manifest)
        .expect("Could not write Package.swift!");

    create_dir_all(format!("{}/Sources/{}", package_name, package_name))
        .expect("Could not create module sources directory!");

    copy(
        "./generated/sources/lib.swift",
        format!("{}/Sources/{}/lib.swift", package_name, package_name),
    )
    .expect("Could not copy generated swift source files!");
}

pub fn recreate_output_dir(package_name: &str) -> io::Result<()> {
    let dir = format!("./{package_name}");

    recreate_dir(dir)
}
