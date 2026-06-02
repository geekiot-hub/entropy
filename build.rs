fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/entropy.ico");
    res.set("ProductName", "Entropy");
    res.set("FileDescription", "Entropy");
    res.set("CompanyName", "Ergohaven");
    res.set("LegalCopyright", "© Ergohaven");
    if let Err(err) = res.compile() {
        eprintln!("failed to embed Windows resources: {err}");
    }
}
