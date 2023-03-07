use std::process::Command;

use execute::command;

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
        architectures: Vec<&'static str>,
        display_name: &'static str,
        platform: ApplePlatform,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Debug,
    Release,
}

impl Target {
    fn cargo_build_commands(&self, mode: Mode) -> Vec<Command> {
        let flag = match mode {
            Mode::Debug => "",
            Mode::Release => "--release",
        };

        self.architectures()
            .into_iter()
            .map(|arch| command(format!("cargo build --target {arch} {flag}")))
            .collect()
    }

    fn lipo_commands(&self, lib_name: &str, mode: Mode) -> Vec<Command> {
        let mode_str = match mode {
            Mode::Debug => "debug",
            Mode::Release => "release",
        };

        match self {
            Target::Single { .. } => vec![],
            Target::Universal { architectures, .. } => {
                let path = self.library_directory(mode);

                let target_name = format!("lib{}.dylib", lib_name);
                let component_paths: Vec<_> = architectures
                    .iter()
                    .map(|arch| format!("./target/{arch}/{mode_str}/{target_name}"))
                    .collect();
                let args = component_paths.join(" ");
                let target_path = self.library_file(lib_name, mode);

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
    pub fn commands(&self, lib_name: &str, mode: Mode) -> Vec<Command> {
        self.cargo_build_commands(mode)
            .into_iter()
            .chain(self.lipo_commands(lib_name, mode))
            .collect()
    }

    /// Returns the names of all target architectures for this target
    ///
    /// If this target is a single target, the returned vector will always contain exactly one element.
    /// The names returned here exactly match the identifiers of the respective official Rust targets.
    pub fn architectures(&self) -> Vec<&'static str> {
        match self {
            Target::Single { architecture, .. } => vec![architecture],
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

        match self {
            Target::Single { architecture, .. } => format!("./target/{architecture}/{mode}"),
            Target::Universal { universal_name, .. } => format!("./target/{universal_name}/{mode}"),
        }
    }

    pub fn library_file(&self, lib_name: &str, mode: Mode) -> String {
        format!("{}/lib{}.dylib", self.library_directory(mode), lib_name)
    }
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
                architectures: vec!["x86_64-apple-ios", "aarch64-apple-ios-sim"],
                display_name: "iOS Simulator",
                platform: *self,
            },
            MacOS => Target::Universal {
                universal_name: "universal-macos",
                architectures: vec!["x86_64-apple-darwin", "aarch64-apple-darwin"],
                display_name: "macOS",
                platform: *self,
            },
            MacCatalyst => {
                unimplemented!("No official Rust target for platform \"Mac Catalyst\"!")
            }
            TvOS => Target::Universal {
                universal_name: "universal-tvos",
                architectures: vec!["aarch64-apple-tvos", "x86_64-apple-tvos"],
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
            }
        }
    }
}
