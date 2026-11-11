fn main() {
    let icon_path = "src/assets/icon.ico";
    println!("cargo:rerun-if-changed={icon_path}");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winres::WindowsResource::new()
            .set_icon(icon_path)
            .compile()
            .expect("failed to embed Windows application icon");
    }
}
