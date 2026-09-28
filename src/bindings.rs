use std::{
    fs::{self, create_dir},
};

use anyhow::anyhow;
use crate::Result;
use camino::Utf8Path;
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage};

use crate::recreate_dir;

/// Generates UniFFI bindings for the crate and returns the primary FFI module name.
///
/// A library may comprise several UniFFI components — multiple crates that each call
/// `setup_scaffolding!()` — in which case `uniffi-bindgen --library` emits one FFI header,
/// `module.modulemap`, and Swift file per component. When packaged as a dynamic `.framework`, a
/// bundle only exposes the Clang module whose name matches the bundle, so several `framework
/// module` blocks in one bundle would leave all but one unimportable. To keep every component
/// importable, a single module is emitted that exposes all component headers (UniFFI's
/// `UNIFFI_SHARED_H` / `UNIFFI_FFIDEF_*` include guards deduplicate the shared runtime types) and
/// each generated Swift file's FFI import is repointed at it.
///
/// `lib_name` identifies the primary component, whose FFI module name the framework and
/// xcframework take, so the result does not depend on directory iteration order.
pub fn generate_bindings(lib_path: &Utf8Path, lib_name: &str) -> Result<String> {
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

    // Collect the FFI module name of every generated component (one per `*.h`, excluding the
    // Swift bridging header).
    let mut ffi_modules: Vec<String> = fs::read_dir(out_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "h"))
        .filter_map(|path| {
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .filter(|stem| !stem.contains("BridgingHeader"))
        })
        .collect();
    ffi_modules.sort();
    ffi_modules.dedup();

    if ffi_modules.is_empty() {
        return Err(anyhow!("Could not find generated header file in {}", out_dir).into());
    }

    // The primary FFI module is the one imported by the built crate's own Swift file
    // (`{lib_name}.swift`). Falling back to the first header keeps single-component behaviour.
    let primary = primary_ffi_module(out_dir, lib_name, &ffi_modules)
        .unwrap_or_else(|| ffi_modules[0].clone());

    // Emit one module exposing every component header (see doc comment).
    let mut modulemap = format!("module {primary} {{\n");
    for module in &ffi_modules {
        modulemap.push_str(&format!("    header \"{module}.h\"\n"));
    }
    modulemap.push_str("    export *\n    use \"Darwin\"\n    use \"_Builtin_stdbool\"\n    use \"_Builtin_stdint\"\n}\n");
    fs::write(headers.join("module.modulemap"), modulemap)?;

    // Copy headers and Swift sources into the package layout, repointing every component's FFI
    // import at the single merged module. Per-component `.modulemap` files are ignored — we
    // wrote the merged one above.
    for entry in fs::read_dir(out_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !entry.metadata()?.is_file() {
            continue;
        }
        let Some(name) = path.file_name() else { continue };
        let Some(ext) = path.extension() else { continue };

        if ext == "h" {
            fs::copy(&path, headers.join_os(name))?;
        } else if ext == "swift" {
            let mut content = fs::read_to_string(&path)?;
            for module in &ffi_modules {
                if module == &primary {
                    continue;
                }
                content = content
                    .replace(&format!("canImport({module})"), &format!("canImport({primary})"))
                    .replace(&format!("import {module}\n"), &format!("import {primary}\n"));
            }
            fs::write(sources.join_os(name), content)?;
        }
    }

    Ok(primary)
}

/// Detects the FFI module imported by the built crate's own Swift bindings file.
///
/// UniFFI names that file after the crate's namespace, which defaults to the library name.
/// Returns the imported module only if it is one of the generated FFI modules.
fn primary_ffi_module(out_dir: &Utf8Path, lib_name: &str, ffi_modules: &[String]) -> Option<String> {
    let content = fs::read_to_string(out_dir.join(format!("{lib_name}.swift"))).ok()?;
    content.lines().find_map(|line| {
        let module = line.trim().strip_prefix("import ")?.trim();
        ffi_modules.iter().find(|m| m.as_str() == module).cloned()
    })
}
