use std::fs::{create_dir, create_dir_all, write};

pub fn create_swiftpackage(package_name: &str) {
    create_dir(package_name).expect("Could not create swift package directory!");

    let package_manifest =
        include_str!("../template/template.Package.swift").replace("<PACKAGE_NAME>", package_name);

    write(format!("{}/Package.swift", package_name), package_manifest)
        .expect("Could not write Cargo.toml!");

    create_dir_all(format!("{}/Sources/{}", package_name, package_name))
        .expect("Could not create module sources directory!");
}
