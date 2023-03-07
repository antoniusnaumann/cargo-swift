use std::fs::{create_dir, create_dir_all, remove_dir_all, write};
use std::io;

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
}

pub fn recreate_output_dir(package_name: &str) -> io::Result<()> {
    let dir = format!("./{package_name}");

    match remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => create_dir(&dir),
    }
}
