use std::{
    fs::{self, create_dir},
    io,
};

use crate::{metadata::metadata, Result};
use camino::Utf8Path;
use uniffi_bindgen::{bindings::SwiftBindingGenerator, cargo_metadata::CrateConfigSupplier};

use crate::recreate_dir;

/// Generates UniFFI bindings for crate and returns the .udl namespace
pub fn generate_bindings(lib_path: &Utf8Path) -> Result<()> {
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

    for output in uniffi_outputs {
        let crate_name = output.ci.crate_name();
        fs::copy(
            out_dir.join(format!("{crate_name}.swift")),
            sources.join(format!("{crate_name}.swift")),
        )?;

        let ffi_name = format!("{crate_name}FFI");
        fs::copy(
            out_dir.join(format!("{ffi_name}.h")),
            headers.join(format!("{ffi_name}.h")),
        )?;

        let mut modulemap_part = fs::OpenOptions::new()
            .read(true)
            .open(out_dir.join(format!("{ffi_name}.modulemap")))?;
        io::copy(&mut modulemap_part, &mut modulemap)?;
    }

    Ok(())
}
