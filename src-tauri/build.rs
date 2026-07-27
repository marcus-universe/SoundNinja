fn main() {
    // When `stems` is on, ort's prebuilt C++ objects need the MSVC C++ stdlib.
    // Harmless if the feature is off (cfg gates this out).
    #[cfg(all(windows, feature = "stems"))]
    {
        println!("cargo:rustc-link-lib=dylib=msvcprt");
    }

    tauri_build::build();
}
