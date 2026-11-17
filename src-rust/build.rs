// Nhuong icon vao file .exe khi build cho Windows (can windres cua mingw khi cross-compile).
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("ProductName", "Markdown Tabs");
        res.set("FileDescription", "Markdown Tabs - Markdown viewer & editor");
        res.set("CompanyName", "LisaTools");
        res.set("LegalCopyright", "LisaTools");
        if let Err(e) = res.compile() {
            println!("cargo:warning=winresource failed: {e}");
        }
    }
}
