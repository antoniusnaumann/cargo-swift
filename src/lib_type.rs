use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(PartialEq, Eq, Clone, Copy)]
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

#[derive(Debug, Error)]
pub struct VariantError {
    input: String,
}

impl Display for VariantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Unsupported variant for crate-type: {}",
            &self.input
        ))
    }
}

impl FromStr for LibType {
    type Err = VariantError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "staticlib" => Ok(Self::Static),
            "cdylib" => Ok(Self::Dynamic),
            _ => Err(VariantError {
                input: String::from(s),
            }),
        }
    }
}
