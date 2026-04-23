fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.1".to_string());
    let numeric = parse_version_u64(&version);
    let display = version_string_for_windows(&version);

    let mut res = winres::WindowsResource::new();
    res.set_windres_path("/usr/bin/x86_64-w64-mingw32-windres");
    res.set_ar_path("/usr/bin/x86_64-w64-mingw32-ar");
    res.set("ProductName", "Entropy");
    res.set("FileDescription", "Ergohaven keyboard configurator");
    res.set("OriginalFilename", "entropy.exe");
    res.set("InternalName", "entropy");
    res.set("FileVersion", &display);
    res.set("ProductVersion", &display);
    res.set_version_info(winres::VersionInfo::FILEVERSION, numeric);
    res.set_version_info(winres::VersionInfo::PRODUCTVERSION, numeric);
    res.compile().expect("failed to compile Windows resources");
}

fn version_string_for_windows(version: &str) -> String {
    let mut parts: Vec<u16> = version
        .split('.')
        .filter_map(|part| part.parse::<u16>().ok())
        .collect();
    while parts.len() < 4 {
        parts.push(0);
    }
    format!("{}.{}.{}.{}", parts[0], parts[1], parts[2], parts[3])
}

fn parse_version_u64(version: &str) -> u64 {
    let mut parts: Vec<u16> = version
        .split('.')
        .filter_map(|part| part.parse::<u16>().ok())
        .collect();
    while parts.len() < 4 {
        parts.push(0);
    }

    ((parts[0] as u64) << 48)
        | ((parts[1] as u64) << 32)
        | ((parts[2] as u64) << 16)
        | (parts[3] as u64)
}
