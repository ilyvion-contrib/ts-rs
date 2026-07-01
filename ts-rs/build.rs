fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    println!("cargo:rerun-if-env-changed=TS_RS_EXPORT_DIR");
    println!("cargo:rerun-if-env-changed=TS_RS_IMPORT_EXTENSION");
    println!("cargo:rerun-if-env-changed=TS_RS_LARGE_INT");
}
