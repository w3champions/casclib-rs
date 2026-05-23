extern crate cmake;

use std::env;

fn main() {
    // Gets CacsLib source path from env CASCLIB_DIR
    let casclib_path = env::var("CASCLIB_DIR").unwrap_or("../deps/CascLib".to_string());

    println!("cargo:rerun-if-changed={}", casclib_path);

    let mut cfg = cmake::Config::new(&casclib_path);

    // Gate on the *target* OS, not the host. A build script runs on the
    // host, so `#[cfg(target_os = ...)]` here would silently drop the
    // UNICODE defines when cross-compiling Linux → x86_64-pc-windows-msvc,
    // producing a C library whose ABI disagrees with the (UNICODE)
    // bindings_windows.rs.
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        cfg.cxxflag("-D UNICODE")
            .cxxflag("-D _UNICODE")
            .cxxflag("-D CASC_UNICODE")
            .cxxflag("-D CASCLIB_UNICODE");

        // Match Rust's +crt-static (the default for cargo-xwin builds): the
        // C/C++ objects must use the static CRT (/MT), or the final link
        // fails with __declspec(dllimport) CRT symbols against static
        // libcmt. CascLib's older CMakeLists makes CMAKE_MSVC_RUNTIME_LIBRARY
        // / CMP0091 unreliable, so override the per-config flags directly.
        if env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc" {
            cfg.define("CMAKE_C_FLAGS_RELEASE", "/MT /O2 /Ob2 /DNDEBUG")
                .define("CMAKE_CXX_FLAGS_RELEASE", "/MT /O2 /Ob2 /DNDEBUG");
        }
    }

    // Builds CascLib using cmake
    let dst = cfg
        .define("CASC_BUILD_SHARED_LIB", "OFF")
        .define("CASC_BUILD_STATIC_LIB", "ON")
        .define("CMAKE_POLICY_VERSION_MINIMUM", "3.5")
        // CascLib's CMakeLists pulls in GNUInstallDirs, which resolves
        // CMAKE_INSTALL_LIBDIR to `lib64` on multiarch x86_64 systems. The
        // cmake crate only adds `<install_prefix>/lib` to the rustc link
        // search path, so the static archive becomes unreachable. Force the
        // simple `lib` layout to keep the search path consistent across
        // hosts.
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .build();

    let lib = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=static=casc");

    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=z");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=z");
    }
}
