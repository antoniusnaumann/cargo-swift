use std::{
    fs::{self, create_dir},
    io::{self, Write},
};

use crate::Result;
use camino::Utf8Path;
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage};

use crate::recreate_dir;

/// Generates UniFFI bindings for crate and returns the .udl namespace
pub fn generate_bindings(lib_path: &Utf8Path) -> Result<()> {
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

    Ok(())
}
