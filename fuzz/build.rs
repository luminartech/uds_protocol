fn main() {
    // On Windows MSVC, the linker won't pull `main` from static libraries
    // unless explicitly told to. libfuzzer-sys compiles LLVM's FuzzerMain.cpp
    // (which defines `main`) into `fuzzer.lib`, but since nothing in the Rust
    // code references `main`, MSVC's linker drops it — causing LNK1561.
    //
    // Force the linker to include the `main` symbol from the fuzzer static lib.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg=/INCLUDE:main");

        // cargo fuzz passes -Zsanitizer=address which links against
        // clang_rt.asan_dynamic-x86_64.dll. Windows only searches the exe's
        // directory and PATH for DLLs. Locate the DLL from the MSVC toolchain
        // and copy it next to the output binary so it's found at runtime.
        if let Some(dll_path) = find_asan_dll() {
            let out_dir = std::env::var("OUT_DIR").unwrap();
            // OUT_DIR is deep inside target/; walk up to the profile directory
            // (e.g. target/x86_64-pc-windows-msvc/release/)
            if let Some(profile_dir) = find_profile_dir(&out_dir) {
                let dest =
                    std::path::Path::new(&profile_dir).join("clang_rt.asan_dynamic-x86_64.dll");
                if !dest.exists() {
                    let _ = std::fs::copy(&dll_path, &dest);
                }
            }
        }
    }
}

#[cfg(windows)]
fn find_asan_dll() -> Option<std::path::PathBuf> {
    // Check common MSVC toolchain locations
    let vswhere_paths = [std::env::var("VCToolsInstallDir").ok(), glob_msvc_path()];
    for base in vswhere_paths.into_iter().flatten() {
        let candidate = std::path::PathBuf::from(&base)
            .join("bin/Hostx64/x64/clang_rt.asan_dynamic-x86_64.dll");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    // Fallback: LLVM installation
    for entry in std::fs::read_dir("C:/Program Files/LLVM/lib/clang").ok()? {
        let entry = entry.ok()?;
        let candidate = entry
            .path()
            .join("lib/windows/clang_rt.asan_dynamic-x86_64.dll");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(not(windows))]
fn find_asan_dll() -> Option<std::path::PathBuf> {
    None
}

fn glob_msvc_path() -> Option<String> {
    let base = "C:/Program Files (x86)/Microsoft Visual Studio";
    let vs_dir = std::fs::read_dir(base).ok()?;
    for edition in vs_dir {
        let edition = edition.ok()?;
        for product in std::fs::read_dir(edition.path()).ok()? {
            let product = product.ok()?;
            let tools = product.path().join("VC/Tools/MSVC");
            if let Ok(versions) = std::fs::read_dir(&tools) {
                for ver in versions {
                    let ver = ver.ok()?;
                    return Some(ver.path().to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}

fn find_profile_dir(out_dir: &str) -> Option<String> {
    // OUT_DIR looks like: .../target/<triple>/release/build/<pkg>/out
    // Walk up until we find a directory containing "release" or "debug"
    let mut path = std::path::Path::new(out_dir);
    while let Some(parent) = path.parent() {
        if path
            .file_name()
            .is_some_and(|n| n == "release" || n == "debug")
        {
            return Some(path.to_string_lossy().into_owned());
        }
        path = parent;
    }
    None
}
