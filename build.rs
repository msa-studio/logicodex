// =========================================================================
// Logicodex v1.42 — Build Script: Raylib Detection + Linking
//
// Detects installed Raylib library and configures linking:
//   - Linux:   pkg-config → -lraylib, or fallback to common paths
//   - macOS:   Homebrew /opt/homebrew/lib or /usr/local/lib
//   - Windows: vcpkg or manual RAYLIB_DIR env
//
// Graceful fallback: if Raylib not found, emit warning but continue build.
// The runtime linking test will report the actual link status.
// =========================================================================

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RAYLIB_DIR");
    println!("cargo:rerun-if-env-changed=RAYLIB_NO_LINK");

    // Allow opting out: RAYLIB_NO_LINK=1 skips Raylib detection
    if env::var("RAYLIB_NO_LINK").is_ok() {
        println!("cargo:warning=Raylib linking disabled by RAYLIB_NO_LINK");
        println!("cargo:rustc-cfg=raylib_no_link");
        return;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Try pkg-config first (most common on Linux)
    if try_pkg_config() {
        println!("cargo:rustc-cfg=raylib_linked");
        return;
    }

    // Try RAYLIB_DIR environment variable
    if let Ok(raylib_dir) = env::var("RAYLIB_DIR") {
        let lib_dir = PathBuf::from(&raylib_dir).join("lib");
        if lib_dir.exists() {
            println!("cargo:rustc-link-search=native={}", lib_dir.display());
            println!("cargo:rustc-link-lib=raylib");
            println!("cargo:rustc-cfg=raylib_linked");
            println!("cargo:warning=Raylib found via RAYLIB_DIR={}", raylib_dir);
            return;
        }
    }

    // Platform-specific fallback paths
    match target_os.as_str() {
        "linux" => {
            let candidates = [
                "/usr/lib/x86_64-linux-gnu",
                "/usr/lib64",
                "/usr/lib",
                "/usr/local/lib",
            ];
            for path in &candidates {
                if PathBuf::from(path).join("libraylib.so").exists()
                    || PathBuf::from(path).join("libraylib.a").exists()
                {
                    println!("cargo:rustc-link-search=native={}", path);
                    println!("cargo:rustc-link-lib=raylib");
                    println!("cargo:rustc-cfg=raylib_linked");
                    println!("cargo:warning=Raylib found at {}", path);
                    return;
                }
            }
        }
        "macos" => {
            let candidates = [
                "/opt/homebrew/lib", // Apple Silicon
                "/usr/local/lib",    // Intel Mac
            ];
            for path in &candidates {
                if PathBuf::from(path).join("libraylib.dylib").exists()
                    || PathBuf::from(path).join("libraylib.a").exists()
                {
                    println!("cargo:rustc-link-search=native={}", path);
                    println!("cargo:rustc-link-lib=raylib");
                    println!("cargo:rustc-cfg=raylib_linked");
                    println!("cargo:warning=Raylib found at {}", path);
                    return;
                }
            }
        }
        "windows" => {
            // vcpkg integration handled by vcpkg crate if needed
            println!("cargo:warning=Windows: Set RAYLIB_DIR env var to Raylib installation path");
        }
        _ => {}
    }

    // Graceful fallback: emit warning but don't fail the build
    println!("cargo:warning=Raylib library not found. Install it for full FFI linking:");
    println!("cargo:warning=  Ubuntu/Debian: sudo apt install libraylib-dev");
    println!("cargo:warning=  Fedora:       sudo dnf install raylib-devel");
    println!("cargo:warning=  macOS:        brew install raylib");
    println!("cargo:warning=  Or set RAYLIB_DIR env var to the Raylib installation");
    println!("cargo:warning=  Or set RAYLIB_NO_LINK=1 to skip Raylib detection");
    println!("cargo:rustc-cfg=raylib_no_link");
}

fn try_pkg_config() -> bool {
    if let Ok(output) = std::process::Command::new("pkg-config")
        .args(["--exists", "raylib"])
        .output()
    {
        if output.status.success() {
            // Get library flags
            if let Ok(output) = std::process::Command::new("pkg-config")
                .args(["--libs", "raylib"])
                .output()
            {
                let libs = String::from_utf8_lossy(&output.stdout);
                for lib in libs.split_whitespace() {
                    if lib.starts_with("-L") {
                        println!("cargo:rustc-link-search=native={}", &lib[2..]);
                    } else if lib.starts_with("-l") {
                        println!("cargo:rustc-link-lib={}", &lib[2..]);
                    }
                }
            }
            println!("cargo:warning=Raylib found via pkg-config");
            return true;
        }
    }
    false
}
