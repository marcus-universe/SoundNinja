fn main() {
    // Stem engine uses ort with `load-dynamic` — no static msvcprt link needed.
    tauri_build::build();
}
