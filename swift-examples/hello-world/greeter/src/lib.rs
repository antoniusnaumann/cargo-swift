uniffi::include_scaffolding!("lib");

pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
