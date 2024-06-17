use std::borrow::Cow;

use camino::Utf8Path;
use cargo_metadata::{Metadata, MetadataCommand, Package};
use itertools::Itertools;
use lazy_static::lazy_static;

use crate::path::PathExt;

pub(crate) fn metadata() -> &'static Metadata {
    lazy_static! {
        static ref METADATA: Metadata = MetadataCommand::new()
            .no_deps()
            .other_options(["--offline".to_string()])
            .exec()
            // TODO: Error handling
            .unwrap();
    }

    &METADATA
}

pub(crate) trait MetadataExt {
    fn target_dir(&self) -> Cow<Utf8Path>;
    fn current_crate(&self) -> Option<&Package>;
}

impl MetadataExt for Metadata {
    fn target_dir(&self) -> Cow<Utf8Path> {
        let target_dir = self.target_directory.as_path();
        let relative = target_dir.to_relative();

        match relative {
            Ok(relative) => Cow::from(relative),
            Err(_) => Cow::from(target_dir),
        }
    }

    /// Returns the package metadata for the crate currently at or above the current working directory.
    fn current_crate(&self) -> Option<&Package> {
        let cwd = std::env::current_dir().unwrap();

        self.workspace_packages()
            .into_iter()
            .filter_map(|p| {
                let parent = p
                    .manifest_path
                    .parent()
                    .expect("The Cargo.toml path should end with /Cargo.toml");

                if cwd.starts_with(parent) {
                    Some((p, parent))
                } else {
                    None
                }
            })
            .find_or_first(|(_, parent)| parent.starts_with(&cwd))
            .map(|(package, _)| package)
    }
}
