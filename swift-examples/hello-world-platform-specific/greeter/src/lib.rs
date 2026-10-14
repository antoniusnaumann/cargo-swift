uniffi::setup_scaffolding!();

// Items exported on every platform.

#[derive(uniffi::Enum)]
pub enum Platform {
    Ios,
    MacOs,
}

#[uniffi::export]
pub fn current_platform() -> Platform {
    if cfg!(target_os = "ios") {
        Platform::Ios
    } else {
        Platform::MacOs
    }
}

#[uniffi::export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

// Items exported only on iOS. cargo swift generates bindings for each platform separately,
// so these are only available in Swift when building for iOS.
#[cfg(target_os = "ios")]
mod ios {
    #[derive(uniffi::Record)]
    pub struct HapticPattern {
        pub intensity: f32,
        pub duration_ms: u32,
    }

    #[uniffi::export]
    pub fn default_haptic_pattern() -> HapticPattern {
        HapticPattern {
            intensity: 0.8,
            duration_ms: 120,
        }
    }
}

// Items exported only on macOS.
#[cfg(target_os = "macos")]
mod macos {
    #[derive(uniffi::Enum)]
    pub enum DockBadge {
        None,
        Count { value: u32 },
    }

    #[derive(uniffi::Object)]
    pub struct MenuBarItem {
        title: String,
    }

    #[uniffi::export]
    impl MenuBarItem {
        #[uniffi::constructor]
        pub fn new(title: String) -> Self {
            Self { title }
        }

        pub fn title(&self) -> String {
            self.title.clone()
        }

        pub fn badge(&self, unread: u32) -> DockBadge {
            match unread {
                0 => DockBadge::None,
                value => DockBadge::Count { value },
            }
        }
    }
}
