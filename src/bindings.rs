use anyhow::Error;
use camino::Utf8Path;

pub fn generate_bindings() -> Result<(), Error> {
    // TODO: Walk src directory for udl files
    let udl_file = Utf8Path::new("./src/ffi.udl");
    // TODO: Allow setting a base path here
    let out_dir = Utf8Path::new("./generated");

    uniffi_bindgen::generate_bindings(udl_file, None, vec!["swift"], Some(out_dir), None, true)
}
