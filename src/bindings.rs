use std::fs::{self, create_dir};

use crate::Result;
use camino::Utf8Path;
use uniffi_bindgen::bindings::TargetLanguage;

use crate::recreate_dir;

/// Generates UniFFI bindings for crate and returns the .udl namespace
pub fn generate_bindings(lib_path: &Utf8Path, crate_name: &str) -> Result<()> {
    let out_dir = Utf8Path::new("./generated");
    let headers = out_dir.join("headers");
    let sources = out_dir.join("sources");

    recreate_dir(out_dir)?;
    create_dir(&headers)?;
    create_dir(&sources)?;

    uniffi_bindgen::library_mode::generate_bindings(
        lib_path,
        Some(crate_name.to_owned()),
        &[TargetLanguage::Swift],
        None,
        out_dir,
        false,
    )?;

    fs::copy(
        out_dir.join(format!("{crate_name}.swift")),
        sources.join(format!("{crate_name}.swift")),
    )?;

    let header = format!("{crate_name}FFI.h");
    fs::copy(out_dir.join(&header), headers.join(&header))?;
    fs::copy(
        out_dir.join(format!("{crate_name}FFI.modulemap")),
        headers.join("module.modulemap"),
    )?;

    Ok(())
}
