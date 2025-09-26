fn main() {
    #[cfg(feature = "uniffi")]
    uniffi::generate_scaffolding("src/baad_core.udl").unwrap();
}