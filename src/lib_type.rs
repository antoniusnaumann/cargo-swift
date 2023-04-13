#[derive(Clone, Copy)]
pub enum LibType {
    Static,
    Dynamic,
}

impl LibType {
    /// The identifier used in the crate-type field in Cargo.toml
    pub fn identifier(&self) -> &str {
        match self {
            LibType::Static => "staticlib",
            LibType::Dynamic => "cdylib",
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            LibType::Static => "a",
            LibType::Dynamic => "dylib",
        }
    }
}
