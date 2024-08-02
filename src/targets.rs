use std::{fmt::Display, process::Command};

use execute::command;
use nonempty::{nonempty, NonEmpty};

use crate::lib_type::LibType;
use crate::metadata::{metadata, MetadataExt};
use crate::package::FeatureOptions;

pub trait TargetInfo {
    fn target(&self) -> Target;
}

#[derive(Debug, Clone)]
pub enum Target {
    Single {
        architecture: &'static str,
        display_name: &'static str,
        platform: ApplePlatform,
    },
    Universal {
        universal_name: &'static str,
        architectures: NonEmpty<&'static str>,
        display_name: &'static str,
        platform: ApplePlatform,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Debug,
    Release,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Debug => write!(f, "debug"),
            Mode::Release => write!(f, "release"),
        }
    }
}

impl Target {
    fn cargo_build_commands(&self, mode: Mode, features: &FeatureOptions) -> Vec<Command> {
        self.architectures()
            .into_iter()
            .map(|arch| {
                let mut cmd = command("cargo build");
                cmd.arg("--target").arg(arch);

                match mode {
                    Mode::Debug => {}
                    Mode::Release => {
                        cmd.arg("--release");
                    }
                }

                if let Some(features) = &features.features {
                    cmd.arg("--features").arg(features.join(","));
                }
                if features.all_features {
                    cmd.arg("--all-features");
                }
                if features.no_default_features {
                    cmd.arg("--no-default-features");
                }

                cmd
            })
            .collect()
    }

    fn lipo_commands(&self, lib_name: &str, mode: Mode, lib_type: LibType) -> Vec<Command> {
        match self {
            Target::Single { .. } => vec![],
            Target::Universal { architectures, .. } => {
                let path = self.library_directory(mode);

                let target = metadata().target_dir();
                let target_name = library_file_name(lib_name, lib_type);
                let component_paths: Vec<_> = architectures
                    .iter()
                    .map(|arch| format!("{target}/{arch}/{mode}/{target_name}"))
                    .collect();
                let args = component_paths.join(" ");
                let target_path = self.library_path(lib_name, mode, lib_type);

                let make_dir = command(format!("mkdir -p {path}"));
                let lipo = command(format!("lipo {args} -create -output {target_path}"));
                vec![make_dir, lipo]
            }
        }
    }

    fn rpath_install_id_commands(
        &self,
        lib_name: &str,
        mode: Mode,
        lib_type: LibType,
    ) -> Vec<Command> {
        if matches!(lib_type, LibType::Dynamic) {
            vec![command(format!(
                "install_name_tool -id @rpath/{} {}",
                library_file_name(lib_name, lib_type),
                self.library_path(lib_name, mode, lib_type)
            ))]
        } else {
            vec![]
        }
    }

    /// Generates all commands necessary to build this target
    ///
    /// This function returns a list of commands that should be executed in their given
    /// order to build this target (and bundle architecture targets with lipo if it is a universal target).
    pub fn commands(
        &self,
        lib_name: &str,
        mode: Mode,
        lib_type: LibType,
        features: &FeatureOptions,
    ) -> Vec<Command> {
        self.cargo_build_commands(mode, features)
            .into_iter()
            .chain(self.lipo_commands(lib_name, mode, lib_type))
            .chain(self.rpath_install_id_commands(lib_name, mode, lib_type))
            .collect()
    }

    /// Returns the names of all target architectures for this target
    ///
    /// If this target is a single target, the returned vector will always contain exactly one element.
    /// The names returned here exactly match the identifiers of the respective official Rust targets.
    pub fn architectures(&self) -> NonEmpty<&'static str> {
        match self {
            Target::Single { architecture, .. } => nonempty![architecture],
            Target::Universal { architectures, .. } => architectures.to_owned(),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Target::Single { display_name, .. } => display_name,
            Target::Universal { display_name, .. } => display_name,
        }
    }

    pub fn platform(&self) -> ApplePlatform {
        match self {
            Target::Single { platform, .. } => *platform,
            Target::Universal { platform, .. } => *platform,
        }
    }

    pub fn library_directory(&self, mode: Mode) -> String {
        let mode = match mode {
            Mode::Debug => "debug",
            Mode::Release => "release",
        };

        let target = metadata().target_dir();

        match self {
            Target::Single { architecture, .. } => format!("{target}/{architecture}/{mode}"),
            Target::Universal { universal_name, .. } => format!("{target}/{universal_name}/{mode}"),
        }
    }

    pub fn library_path(&self, lib_name: &str, mode: Mode, lib_type: LibType) -> String {
        format!(
            "{}/{}",
            self.library_directory(mode),
            library_file_name(lib_name, lib_type)
        )
    }
}

pub fn library_file_name(lib_name: &str, lib_type: LibType) -> String {
    format!("lib{}.{}", lib_name, lib_type.file_extension())
}

#[derive(Clone, Copy, Debug)]
pub enum ApplePlatform {
    IOS,
    IOSSimulator,
    MacOS,
    MacCatalyst,
    TvOS,
    WatchOS,
    WatchOSSimulator,
    CarPlayOS,
    CarPlayOSSimulator,
    VisionOS,
    VisionOSSimulator,
}

impl TargetInfo for ApplePlatform {
    fn target(&self) -> Target {
        use ApplePlatform::*;
        match self {
            IOS => Target::Single {
                architecture: "aarch64-apple-ios",
                display_name: "iOS",
                platform: *self,
            },
            IOSSimulator => Target::Universal {
                universal_name: "universal-ios",
                architectures: nonempty!["x86_64-apple-ios", "aarch64-apple-ios-sim"],
                display_name: "iOS Simulator",
                platform: *self,
            },
            MacOS => Target::Universal {
                universal_name: "universal-macos",
                architectures: nonempty!["x86_64-apple-darwin", "aarch64-apple-darwin"],
                display_name: "macOS",
                platform: *self,
            },
            MacCatalyst => {
                unimplemented!("No official Rust target for platform \"Mac Catalyst\"!")
            }
            TvOS => Target::Universal {
                universal_name: "universal-tvos",
                architectures: nonempty!["aarch64-apple-tvos", "x86_64-apple-tvos"],
                display_name: "tvOS",
                platform: *self,
            },
            WatchOS => {
                unimplemented!("No official Rust target for platform \"watchOS\"!")
            }
            WatchOSSimulator => {
                unimplemented!("No official Rust target for platform \"watchOS Simulator\"!")
            }
            CarPlayOS => unimplemented!("No official Rust target for platform \"CarPlay\"!"),
            CarPlayOSSimulator => {
                unimplemented!("No official Rust target for platform \"CarPlay Simulator\"!")
            },
            VisionOS => unimplemented!(),
            VisionOSSimulator => unimplemented!(),
        }
    }
}
