uniffi::setup_scaffolding!();

#[derive(uniffi::Object)]
pub struct Greeter {}

#[uniffi::export]
impl Greeter {
    pub fn greet(&self, name: String) -> String {
        format!("Hello, {}!", name)
    }
}
