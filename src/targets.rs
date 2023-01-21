use std::process::Command;

use execute::command;
use swift_bridge_build::ApplePlatform;

pub trait TargetInfo {
    fn target(&self) -> Target;
}

#[derive(Debug, Clone)]
pub enum Target {
    Single {
        architecture: &'static str,
    },
    Universal {
        universal_name: &'static str,
        architectures: Vec<&'static str>,
    },
}

impl Target {
    fn cargo_build_commands(&self) -> Vec<Command> {
        self.architectures()
            .into_iter()
            .map(|arch| command(format!("cargo build --target {arch}")))
            .collect()
    }

    fn lipo_commands(&self, crate_name: &str) -> Vec<Command> {
        // TODO: Make this configurable
        let mode = "debug";
        match self {
            Target::Single { architecture: _ } => vec![],
            Target::Universal {
                universal_name,
                architectures,
            } => {
                let path = format!("./target/{universal_name}/{mode}");

                let target_name = format!("lib{}.a", crate_name.replace('-', "_"));
                let component_paths: Vec<_> = architectures
                    .into_iter()
                    .map(|arch| format!("./target/{arch}/{mode}/{target_name}"))
                    .collect();
                let args = component_paths.join(" ");
                let target_path = format!("{path}/{target_name}");

                let make_dir = command(format!("mkdir -p {path}"));
                let lipo = command(format!("lipo {args} -create -output {target_path}"));
                vec![make_dir, lipo]
            }
        }
    }

    /// Generates all commands necessary to build this target
    ///
    /// This function returns a list of commands that should be executed in their given
    /// order to build this target (and bundle architecture targets with lipo if it is a universal target).
    pub fn commands(&self, crate_name: &str) -> Vec<Command> {
        self.cargo_build_commands()
            .into_iter()
            .chain(self.lipo_commands(crate_name))
            .collect()
    }

    /// Returns the names of all target architectures for this target
    ///
    /// If this target is a single target, the returned vector will always contain exactly one element.
    /// The names returned here exactly match the identifiers of the respective official Rust targets.
    pub fn architectures(&self) -> Vec<&'static str> {
        match self {
            Target::Single { architecture } => vec![architecture],
            Target::Universal {
                universal_name,
                architectures,
            } => architectures.to_owned(),
        }
    }
}

impl TargetInfo for ApplePlatform {
    fn target(&self) -> Target {
        use ApplePlatform::*;
        match self {
            IOS => Target::Single {
                architecture: "aarch64-apple-ios",
            },
            Simulator => Target::Universal {
                universal_name: "universal-ios",
                architectures: vec!["x86_64-apple-ios", "aarch64-apple-ios-sim"],
            },
            MacOS => Target::Universal {
                universal_name: "universal-macos",
                architectures: vec!["x86_64-apple-darwin", "aarch64-apple-darwin"],
            },
            MacCatalyst => {
                unimplemented!("No official Rust target for platform \"Mac Catalyst\"!")
            }
            TvOS => Target::Universal {
                universal_name: "universal-tvos",
                architectures: vec!["aarch64-apple-tvos", "x86_64-apple-tvos"],
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
            }
        }
    }
}
