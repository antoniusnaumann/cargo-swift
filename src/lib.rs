#![allow(clippy::useless_format)]

mod command;
mod commands {
    pub mod init;
    pub mod package;
}

mod bindings;
mod config;
mod spinners;
mod swiftpackage;
mod targets;
mod xcframework;

pub use command::*;
pub use commands::*;
pub use config::*;
pub use targets::*;
