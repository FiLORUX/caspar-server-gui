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
    // Try to read from registry
    // HKEY_LOCAL_MACHINE\SOFTWARE\NDI\InstallDir
    // Then check version from NDI SDK DLL

    // For now, check if NDI SDK environment variable exists
    std::env::var("NDI_SDK_DIR")
        .ok()
        .map(|_| "Installed".to_string())
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
