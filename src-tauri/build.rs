use std::env;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // Only build DeckLink wrapper when the feature is enabled
    #[cfg(feature = "decklink")]
    build_decklink_wrapper();
}

#[cfg(feature = "decklink")]
fn build_decklink_wrapper() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("sdk/decklink");
    let include_dir = sdk_dir.join("include");
    let wrapper_dir = sdk_dir.join("wrapper");

    println!("cargo:rerun-if-changed=sdk/decklink/wrapper/decklink_wrapper.cpp");
    println!("cargo:rerun-if-changed=sdk/decklink/wrapper/decklink_wrapper.h");
    println!("cargo:rerun-if-changed=sdk/decklink/include/DeckLinkAPI.idl");

    if target_os == "windows" {
        // On Windows, we need to compile IDL files with MIDL first
        compile_idl_files(&include_dir, &out_dir);

        // Then compile the C++ wrapper
        cc::Build::new()
            .cpp(true)
            .file(wrapper_dir.join("decklink_wrapper.cpp"))
            .include(&include_dir)
            .include(&out_dir) // For generated headers from MIDL
            .include(&wrapper_dir)
            .define("_WIN32", None)
            .define("WIN32_LEAN_AND_MEAN", None)
            .flag_if_supported("/std:c++17")
            .flag_if_supported("/EHsc")
            .compile("decklink_wrapper");

        // Link against Windows libraries needed for COM
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=oleaut32");
    } else {
        // On non-Windows platforms, compile a stub implementation
        cc::Build::new()
            .cpp(true)
            .file(wrapper_dir.join("decklink_wrapper.cpp"))
            .include(&wrapper_dir)
            .flag_if_supported("-std=c++17")
            .compile("decklink_wrapper");
    }
}

#[cfg(feature = "decklink")]
fn compile_idl_files(include_dir: &PathBuf, out_dir: &PathBuf) {
    use std::process::Command;

    let idl_file = include_dir.join("DeckLinkAPI.idl");

    if !idl_file.exists() {
        panic!(
            "DeckLinkAPI.idl not found at {:?}. Please copy the DeckLink SDK files.",
            idl_file
        );
    }

    // Find MIDL compiler (usually in Windows SDK)
    let midl = find_midl();

    println!("cargo:warning=Running MIDL compiler: {:?}", midl);
    println!("cargo:warning=IDL file: {:?}", idl_file);
    println!("cargo:warning=Output dir: {:?}", out_dir);

    let status = Command::new(&midl)
        .arg(&idl_file)
        .arg("/h")
        .arg(out_dir.join("DeckLinkAPI_h.h"))
        .arg("/iid")
        .arg(out_dir.join("DeckLinkAPI_i.c"))
        .arg("/tlb")
        .arg(out_dir.join("DeckLinkAPI.tlb"))
        .arg("/I")
        .arg(include_dir)
        .arg("/nologo")
        .arg("/W1")
        .arg("/char")
        .arg("signed")
        .arg("/env")
        .arg("x64")
        .current_dir(include_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=MIDL compilation successful");
        }
        Ok(s) => {
            panic!("MIDL compilation failed with exit code: {:?}", s.code());
        }
        Err(e) => {
            panic!(
                "Failed to run MIDL compiler at {:?}: {}. \
                 Make sure Windows SDK is installed and MIDL is in PATH, \
                 or set MIDL_PATH environment variable.",
                midl, e
            );
        }
    }
}

#[cfg(all(feature = "decklink", target_os = "windows"))]
fn find_midl() -> PathBuf {
    // Check environment variable first
    if let Ok(path) = env::var("MIDL_PATH") {
        return PathBuf::from(path);
    }

    // Try to find MIDL in common Windows SDK locations
    let sdk_paths = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];

    for sdk_path in &sdk_paths {
        let sdk_dir = PathBuf::from(sdk_path);
        if sdk_dir.exists() {
            // Find the latest SDK version directory
            if let Ok(entries) = std::fs::read_dir(&sdk_dir) {
                let mut versions: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .map(|s| s.starts_with("10."))
                            .unwrap_or(false)
                    })
                    .collect();

                versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                if let Some(latest) = versions.first() {
                    let midl_path = latest.path().join("x64").join("midl.exe");
                    if midl_path.exists() {
                        return midl_path;
                    }
                }
            }
        }
    }

    // Fall back to PATH
    PathBuf::from("midl.exe")
}

#[cfg(all(feature = "decklink", not(target_os = "windows")))]
fn find_midl() -> PathBuf {
    // On non-Windows, MIDL isn't available
    // This function should never be called on non-Windows platforms
    // because compile_idl_files is only called on Windows
    PathBuf::from("midl")
}
