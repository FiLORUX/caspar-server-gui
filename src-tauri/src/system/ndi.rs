// NDI Tools version detection
// Checks for installed NDI software

use std::process::Command;

/// Get installed NDI Tools version
///
/// Detection method varies by platform:
/// - Windows: Registry or NDI SDK DLL version
/// - macOS: Application bundle version
/// - Linux: Library version in /usr/lib
pub fn get_ndi_version() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        get_ndi_version_windows()
    }

    #[cfg(target_os = "macos")]
    {
        get_ndi_version_macos()
    }

    #[cfg(target_os = "linux")]
    {
        get_ndi_version_linux()
    }
}

#[cfg(target_os = "windows")]
fn get_ndi_version_windows() -> Option<String> {
    use std::path::Path;

    const DLL: &str = "Processing.NDI.Lib.x64.dll";

    // NDI Tools/Runtime set a versioned env var pointing at the runtime directory
    // that holds the loader DLL (this is exactly what CasparCG loads). Check the
    // newest first, then the SDK variable.
    for var in [
        "NDI_RUNTIME_DIR_V6",
        "NDI_RUNTIME_DIR_V5",
        "NDI_RUNTIME_DIR_V4",
        "NDI_SDK_DIR",
    ] {
        if let Ok(dir) = std::env::var(var) {
            if Path::new(&dir).join(DLL).exists() || Path::new(&dir).exists() {
                return Some(format!("Installed ({})", var.trim_start_matches("NDI_")));
            }
        }
    }

    // Fall back to scanning the standard install root. NDI Tools installs as e.g.
    // "C:\Program Files\NDI\NDI 6 Tools\Runtime\Processing.NDI.Lib.x64.dll";
    // runtimes as "...\NDI 6 Runtime\v6\...". Report the product folder name.
    let root = Path::new(r"C:\Program Files\NDI");
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let dir = entry.path();
            for sub in ["", "Runtime", "v6", "v5", "v4", r"Bin\x64"] {
                let dll = if sub.is_empty() {
                    dir.join(DLL)
                } else {
                    dir.join(sub).join(DLL)
                };
                if dll.exists() {
                    return dir
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(str::to_string)
                        .or_else(|| Some("Installed".to_string()));
                }
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn get_ndi_version_macos() -> Option<String> {
    // Check for NDI Video Monitor app or NDI Tools
    let paths = [
        "/Applications/NDI Video Monitor.app",
        "/Applications/NDI Tools/NDI Video Monitor.app",
        "/Library/NDI SDK for Apple",
    ];

    for path in &paths {
        if std::path::Path::new(path).exists() {
            // Try to get version from Info.plist
            if path.ends_with(".app") {
                let plist_path = format!("{}/Contents/Info.plist", path);
                if let Ok(output) = Command::new("defaults")
                    .args(["read", &plist_path, "CFBundleShortVersionString"])
                    .output()
                {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        return Some(version.trim().to_string());
                    }
                }
            }
            return Some("Installed".to_string());
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn get_ndi_version_linux() -> Option<String> {
    // Check for NDI library
    let lib_paths = [
        "/usr/lib/libndi.so",
        "/usr/lib64/libndi.so",
        "/usr/local/lib/libndi.so",
    ];

    for path in &lib_paths {
        if std::path::Path::new(path).exists() {
            // Try to get version from ldconfig
            if let Ok(output) = Command::new("ldconfig")
                .args(["-p"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("libndi") {
                    return Some("Installed".to_string());
                }
            }
            return Some("Installed".to_string());
        }
    }

    // Check NDI environment variable
    std::env::var("NDI_RUNTIME_DIR_V5")
        .or_else(|_| std::env::var("NDI_RUNTIME_DIR_V4"))
        .ok()
        .map(|_| "Installed".to_string())
}

/// Check if NDI is available on this system
pub fn is_ndi_available() -> bool {
    get_ndi_version().is_some()
}
