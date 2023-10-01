uniffi::include_scaffolding!("lib");

#[uniffi::export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
