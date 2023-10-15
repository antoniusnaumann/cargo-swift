use std::ops::Deref;

use camino::{FromPathBufError, Utf8Path, Utf8PathBuf};

use crate::Result;

pub(crate) trait PathExt {
    fn to_relative(&self) -> Result<Utf8PathBuf>;
    fn find_common_path(&self, other: &Utf8Path) -> Utf8PathBuf;
}

impl PathExt for Utf8Path {
    fn to_relative(&self) -> Result<Utf8PathBuf> {
        let cwd = std::env::current_dir()?;
        let cwd: Utf8PathBuf = cwd.try_into().map_err(|e: FromPathBufError| {
            format!(
                "Current working directory is not a valid UTF-8 path: {}",
                e.into_path_buf().to_string_lossy()
            )
        })?;
        let common = self.find_common_path(&cwd);
        let remaining = cwd.strip_prefix(common.deref()).unwrap();
        let prefix = remaining
            .components()
            .map(|_| "..")
            .collect::<Utf8PathBuf>();

        let relative = prefix.join(self.strip_prefix(common).unwrap());

        Ok(relative)
    }

    fn find_common_path(&self, other: &Utf8Path) -> Utf8PathBuf {
        let mut self_components = self.components();
        let mut other_components = other.components();
        let mut common_path = Utf8PathBuf::new();
        while let (Some(self_component), Some(other_component)) =
            (self_components.next(), other_components.next())
        {
            if self_component == other_component {
                common_path.push(self_component);
            } else {
                break;
            }
        }

        common_path
    }
}
