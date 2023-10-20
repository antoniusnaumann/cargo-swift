uniffi::include_scaffolding!("greeter");

pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
