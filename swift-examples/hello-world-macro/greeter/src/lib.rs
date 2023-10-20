uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
