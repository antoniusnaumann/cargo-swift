fn main() {
    uniffi::generate_scaffolding("./src/{{ namespace }}.udl").unwrap();
}
