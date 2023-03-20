use std::fs::{self, create_dir};

use crate::Result;
use camino::Utf8Path;
use regex::Regex;

use crate::recreate_dir;

pub fn generate_bindings() -> Result<String> {
    let udl_file = Utf8Path::new("./src/lib.udl");
    let out_dir = Utf8Path::new("./generated");
    let headers = out_dir.join("headers");
    let sources = out_dir.join("sources");

    recreate_dir(out_dir)?;
    create_dir(&headers)?;
    create_dir(&sources)?;

    uniffi_bindgen::generate_bindings(udl_file, None, vec!["swift"], Some(out_dir), None, false)?;

    let namespace = detect_namespace(udl_file)?;

    fs::copy(
        out_dir.join(format!("{namespace}.swift")),
        sources.join(format!("{namespace}.swift")),
    )?;

    let header = format!("{namespace}FFI.h");
    fs::copy(out_dir.join(&header), headers.join(&header))?;
    fs::copy(
        out_dir.join(format!("{namespace}FFI.modulemap")),
        headers.join("module.modulemap"),
    )?;

    Ok(namespace)
}

fn detect_namespace(udl_file: &Utf8Path) -> Result<String> {
    let content = fs::read_to_string(udl_file)?;

    extract_namespace(&content)
}

fn extract_namespace(content: &str) -> Result<String> {
    let regex = Regex::new(r"namespace\s*([^\{\s]*)\s*\{").unwrap();

    let namespace = regex
        .captures(content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or("lib.udl does not contain a namespace!")?;

    Ok(namespace)
}

#[cfg(test)]
mod tests {
    use super::extract_namespace;

    #[test]
    fn test_extract_namespace_ok() {
        let content = "namespace math { 
            u64 add(u64 a, u64 b);
        };";

        assert_eq!(extract_namespace(content).unwrap(), "math");

        let content = "namespace Example {};";
        assert_eq!(extract_namespace(content).unwrap(), "Example");

        let content = "namespace      whitespace        {}";
        assert_eq!(extract_namespace(content).unwrap(), "whitespace");
    }
}
