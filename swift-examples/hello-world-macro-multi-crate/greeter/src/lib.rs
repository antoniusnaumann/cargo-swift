use greeter_lib::Greeter;

uniffi::setup_scaffolding!();

#[uniffi::export]
pub fn create_greeter() -> Greeter {
    Greeter {}
}
