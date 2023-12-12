{%- if macro_only -%}
uniffi::setup_scaffolding!();
{%- else -%}
uniffi::include_scaffolding!("lib");
{%- endif %}

{% if !plain %}
{% if !macro_only -%}
// Bindings have to be exposed in a .udl file with the same name as the corresponding .rs file, i.e. lib.udl
// You can expose top-level functions...
{%- else -%}
// You can annotate items with uniffi macros to make them available in your swift package.
// You can export functions...
#[uniffi::export]
{%- endif %}
pub fn add(a: u64, b: u64) -> u64 {
    a + b
}

// ... data structs without methods ...
{%- if macro_only ~%}
#[derive(uniffi::Record)]
{%- endif %}
pub struct Example {
    pub items: Vec<String>,
    pub value: Option<f64>,
}

// ... classes with methods ...
{%- if macro_only ~%}
#[derive(uniffi::Object)]
{%- endif %}
pub struct Greeter {
    name: String,
}

{% if macro_only -%}
#[uniffi::export]
{%~ endif -%}
impl Greeter {
    {%- if macro_only %}
    // Constructors need to be annotated as such
    #[uniffi::constructor]
    {%- else %}
    // By convention, a method called new is exposed as a constructor
    {%- endif %}
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
}

{% if macro_only -%}
// ... and much more! For more information about uniffi macros, read the UniFFI book: https://mozilla.github.io/uniffi-rs/proc_macro/index.html
{%- else -%}
// ... and much more! For more information about bindings, read the UniFFI book: https://mozilla.github.io/uniffi-rs/udl_file_spec.html
{%- endif %}
{%- endif %}
