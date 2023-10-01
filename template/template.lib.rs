// Call this for every uniffi module (.udl) you declare
uniffi::include_scaffolding!("lib");

// Bindings have to be exposed in a .udl file with the same name as the corresponding .rs file, i.e. lib.udl
// You can expose top-level functions...
pub fn add(a: u64, b: u64) -> u64 {
    a + b
}

// ... data structs without methods ...
pub struct Example {
    pub items: Vec<String>,
    pub value: Option<f64>,
}

// ... classes with methods ...
pub struct Greeter {
    name: String,
}

impl Greeter {
    // By convention, a method called new is exposed as a constructor
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
}

// Instead of adding bindings in lib.udl, you can also use UniFFIs macros
// Note that you have to keep the .udl file and its namespace if you are using macros
//
// You can read more about UniFFI macros here: https://mozilla.github.io/uniffi-rs/proc_macro/index.html
#[uniffi::export]
pub fn subtract(a: u64, b: u64) -> u64 {
    a - b
}

// ... and much more! For more information about bindings, read the UniFFI book: https://mozilla.github.io/uniffi-rs/udl_file_spec.html
