#![allow(clippy::useless_format)]

pub mod command;
pub mod commands {
    pub mod init;
    pub mod package;
}
pub mod config;
pub mod spinners;
pub mod targets;
pub mod swiftpackage;

pub use command::*;
pub use commands::*;
pub use config::*;
pub use spinners::*;
pub use targets::*;
