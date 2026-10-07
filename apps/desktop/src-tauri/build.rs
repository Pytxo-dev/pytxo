fn main() {
    // generate_context! embeds Vite's hashed assets. Tauri's build helper tracks
    // configuration and capabilities, but not the frontend directory; without
    // this dependency, a frontend-only release can reuse the previous binary.
    println!("cargo:rerun-if-changed=../dist");
    // Tauri's linker-only shim does not recompile native C++ dependencies.
    // Use the consistent Rust/cc/cmake CRT policy from .cargo/windows-msvc.toml.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
        && std::env::var("CARGO_CFG_TARGET_FEATURE")
            .unwrap_or_default()
            .split(',')
            .any(|feature| feature == "crt-static")
    {
        std::env::set_var("STATIC_VCRUNTIME", "false");
    }
    tauri_build::build()
}
