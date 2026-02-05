use std::{
    fs::{self, create_dir},
    io::{self, Write},
};

use anyhow::anyhow;
use crate::Result;
use camino::Utf8Path;
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage};

use crate::recreate_dir;

/// Generates UniFFI bindings for crate and returns the FFI module name.
///
/// This function respects the `ffi_module_name` and `ffi_module_filename` settings
/// in uniffi.toml. The returned FFI module name is detected from the generated
/// header files, which reflect whatever is configured in uniffi.toml.
pub fn generate_bindings(lib_path: &Utf8Path) -> Result<String> {
    let out_dir = Utf8Path::new("./generated");
    let headers = out_dir.join("headers");
    let sources = out_dir.join("sources");

    recreate_dir(out_dir)?;
    create_dir(&headers)?;
    create_dir(&sources)?;

    let options = GenerateOptions {
        languages: vec![TargetLanguage::Swift],
        source: lib_path.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        metadata_no_deps: true,
        ..Default::default()
    };
    uniffi_bindgen::bindings::generate(options)?;

    let mut modulemap = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(headers.join("module.modulemap"))?;

    // Detect the FFI module name from the generated header file.
    // This respects ffi_module_name/ffi_module_filename from uniffi.toml.
    let ffi_module_name = fs::read_dir(out_dir)?
        .filter_map(|entry| entry.ok())
        .find(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "h")
                && entry
                    .path()
                    .file_stem()
                    .is_some_and(|stem| !stem.to_string_lossy().contains("BridgingHeader"))
        })
        .ok_or_else(|| anyhow!("Could not find generated header file in {}", out_dir))?
        .path()
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let dir = fs::read_dir(out_dir)?;

    for f in dir {
        let f = f?;
        let file_path = f.path();

        if !f.metadata()?.is_file() {
            continue;
        }

        let Some(name) = file_path.file_name() else {
            continue;
        };

        let Some(ext) = file_path.extension() else {
            continue;
        };

        if ext == "swift" {
            fs::copy(out_dir.join_os(name), sources.join_os(name))?;
        } else if ext == "h" {
            fs::copy(out_dir.join_os(name), headers.join_os(name))?;
        } else if ext == "modulemap" {
            let mut modulemap_part = fs::OpenOptions::new().read(true).open(file_path)?;
            io::copy(&mut modulemap_part, &mut modulemap)?;
            modulemap.write_all(b"\n")?;
        }
    }

    Ok(ffi_module_name)
}
