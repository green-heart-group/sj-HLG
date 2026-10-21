fn main() {
    println!("cargo:rerun-if-env-changed=HLG_BUILD_FINGERPRINT");
    println!(
        "cargo:rustc-env=HLG_BUILD_FINGERPRINT={}",
        std::env::var("HLG_BUILD_FINGERPRINT").unwrap_or_else(|_| "unrecorded-local-build".into())
    );
    if std::env::var("TARGET").as_deref() == Ok("x86_64-pc-windows-gnu") {
        let path = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("native/windows-gnu");
        println!("cargo:rustc-link-search=native={}", path.display());
        println!("cargo:rerun-if-changed=native/windows-gnu/libimm32.a");
    }
}
