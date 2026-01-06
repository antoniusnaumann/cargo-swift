use std::{
    fs::{self, create_dir},
    io,
};

use crate::{metadata::metadata, Result};
use anyhow::{anyhow, Context};
use camino::Utf8Path;
use uniffi_bindgen::{bindings::SwiftBindingGenerator, cargo_metadata::CrateConfigSupplier};

use crate::recreate_dir;

/// Generates UniFFI bindings for crate and returns the FFI module name
pub fn generate_bindings(
    lib_path: &Utf8Path,
    ffi_module_name_override: Option<&str>,
) -> Result<String> {
    let out_dir = Utf8Path::new("./generated");
    let headers = out_dir.join("headers");
    let sources = out_dir.join("sources");

    recreate_dir(out_dir)?;
    create_dir(&headers)?;
    create_dir(&sources)?;

    let uniffi_outputs = uniffi_bindgen::library_mode::generate_bindings(
        lib_path,
        None,
        &SwiftBindingGenerator {},
        &CrateConfigSupplier::from(metadata().clone()),
        None,
        out_dir,
        false,
    )?;

    let mut modulemap = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(headers.join("module.modulemap"))?;

    let mut final_ffi_name = String::new();

    for output in uniffi_outputs {
        let crate_name = output.ci.crate_name();

        // Find the generated header file to determine the original FFI module name
        let found_ffi_name = fs::read_dir(out_dir)?
            .filter_map(|entry| entry.ok())
            .find(|entry| {
                entry.path().extension().map_or(false, |ext| ext == "h")
                    && entry.path().file_stem().map_or(false, |stem| {
                        !stem.to_string_lossy().contains("BridgingHeader")
                    })
            })
            .ok_or(anyhow!(
                "Could not find generated header file in {}",
                out_dir
            ))?
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Use the override if provided, otherwise use the detected name
        let ffi_name = ffi_module_name_override
            .map(|s| s.to_string())
            .unwrap_or_else(|| found_ffi_name.clone());

        final_ffi_name = ffi_name.clone();

        // Copy and patch the Swift file
        let swift_content = fs::read_to_string(out_dir.join(format!("{crate_name}.swift")))?;
        let patched_swift_content = if found_ffi_name != ffi_name {
            swift_content
                .replace(
                    &format!("import {}", found_ffi_name),
                    &format!("import {}", ffi_name),
                )
                .replace(
                    &format!("canImport({})", found_ffi_name),
                    &format!("canImport({})", ffi_name),
                )
        } else {
            swift_content
        };
        fs::write(
            sources.join(format!("{crate_name}.swift")),
            patched_swift_content,
        )?;

        // Copy the header file (renaming if necessary)
        fs::copy(
            out_dir.join(format!("{found_ffi_name}.h")),
            headers.join(format!("{ffi_name}.h")),
        )?;

        // Copy and patch the modulemap
        let modulemap_content =
            fs::read_to_string(out_dir.join(format!("{found_ffi_name}.modulemap")))?;
        let patched_modulemap_content = if found_ffi_name != ffi_name {
            modulemap_content
                .replace(
                    &format!("module {}", found_ffi_name),
                    &format!("module {}", ffi_name),
                )
                .replace(
                    &format!("header \"{found_ffi_name}.h\""),
                    &format!("header \"{ffi_name}.h\""),
                )
        } else {
            modulemap_content
        };

        use std::io::Write;
        modulemap.write_all(patched_modulemap_content.as_bytes())?;
    }

    Ok(final_ffi_name)
}
