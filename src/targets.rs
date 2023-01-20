use swift_bridge_build::ApplePlatform;

trait TargetInfo {
    fn target(&self) -> Target;
}

#[derive(Debug, Clone)]
enum Target {
    Single {
        architecture: &'static str,
    },
    Universal {
        universal_name: &'static str,
        architectures: Vec<&'static str>,
    },
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
