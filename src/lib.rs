#![allow(clippy::useless_format)]

mod command;
mod commands {
    pub mod init;
    pub mod package;
}

mod bindings;
mod config;
mod error;
mod spinners;
mod swiftpackage;
mod targets;
mod xcframework;

pub use command::*;
pub use commands::*;
pub use config::*;
pub use targets::*;

use std::fs::{create_dir, remove_dir_all};
use std::io;
use std::path::Path;

fn recreate_dir<P>(dir: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    match remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => create_dir(&dir),
    }
}
