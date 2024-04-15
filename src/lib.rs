#![allow(clippy::useless_format)]

mod commands {
    pub mod init;
    pub mod package;
}
pub(crate) mod console {
    mod command;
    pub mod config;
    pub mod error;
    pub mod messages;
    pub mod spinners;
    pub mod step;
    pub mod theme;

    pub use command::*;
    pub use config::*;
    pub use error::*;
    pub use messages::*;
    pub use spinners::*;
    pub use step::*;
    pub use theme::*;
}

mod bindings;
mod lib_type;
mod metadata;
mod path;
mod swiftpackage;
mod targets;
mod templating;
mod xcframework;

pub use crate::console::error::Result;
pub use crate::console::Config;
pub use commands::*;
pub use lib_type::LibType;
pub use targets::*;

use std::fs::{create_dir, remove_dir_all};
use std::io;
use std::path::Path;

fn recreate_dir<P>(dir: P) -> crate::Result<()>
where
    P: AsRef<Path>,
{
    match remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.into()),
        _ => create_dir(&dir).map_err(|e| e.into()),
    }
}
