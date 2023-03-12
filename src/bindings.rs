use std::fs::{self, create_dir};

use crate::Result;
use camino::Utf8Path;

use crate::recreate_dir;

pub fn generate_bindings() -> Result<()> {
    // TODO: Walk src directory for udl files
    let udl_file = Utf8Path::new("./src/lib.udl");
    let out_dir = Utf8Path::new("./generated");
    let headers = out_dir.join("headers");
    let sources = out_dir.join("sources");

    recreate_dir(out_dir)?;
    create_dir(&headers)?;
    create_dir(&sources)?;

    uniffi_bindgen::generate_bindings(udl_file, None, vec!["swift"], Some(out_dir), None, false)?;

    fs::copy(out_dir.join("lib.swift"), sources.join("lib.swift"))?;
    fs::copy(out_dir.join("libFFI.h"), headers.join("libFFI.h"))?;
    fs::copy(
        out_dir.join("libFFI.modulemap"),
        headers.join("module.modulemap"),
    )?;

    Ok(())
}
