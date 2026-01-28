fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_manifest_file("assets/app.manifest");
        res.set_icon("assets/where.ico");
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to embed resources: {}", e);
        }
    }
}
