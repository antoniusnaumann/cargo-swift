use std::fs::{self, create_dir};

use crate::Result;
use camino::Utf8Path;
use uniffi_bindgen::{bindings::TargetLanguage, BindingGeneratorDefault};

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
        &BindingGeneratorDefault {
            target_languages: vec![TargetLanguage::Swift],
            try_format_code: false,
        },
        None,
        out_dir,
        false,
    )?;

    for output in uniffi_outputs {
        let file_name = output.config.module_name();
        fs::copy(
            out_dir.join(format!("{file_name}.swift")),
            sources.join(format!("{file_name}.swift")),
        )?;

        fs::copy(
            out_dir.join(output.config.header_filename()),
            headers.join(output.config.header_filename()),
        )?;
        fs::copy(
            out_dir.join(output.config.modulemap_filename()),
            headers.join(output.config.modulemap_filename()),
        )?;
    }

    Ok(())
}
