extern crate cmake;

use std::env;

fn main() {
    // Gets CacsLib source path from env CASCLIB_DIR
    let casclib_path = env::var("CASCLIB_DIR").unwrap_or("../deps/CascLib".to_string());

    println!("cargo:rerun-if-changed={}", casclib_path);

    let mut cfg = cmake::Config::new(&casclib_path);

    // Rename CascLib's bundled OpenSSL AES symbols to avoid LNK2005 collisions
    // when a downstream crate statically links openssl-sys (e.g. via `vendored`)
    // alongside casclib-sys. CascLib's `deps/CascLib/src/overwatch/aes.cpp` is
    // a copy of OpenSSL's AES implementation with identical exported symbols,
    // so the MSVC linker rejects both archives. We use `-D` preprocessor
    // defines so all translation units (aes.cpp, cmf.cpp, and the aes.h
    // header) see the renamed identifiers without touching the vendored
    // CascLib sources. The rename is harmless on macOS/Linux and keeps the
    // build symmetric across targets.
    cfg.cxxflag("-DAES_set_encrypt_key=CASC_AES_set_encrypt_key")
        .cxxflag("-DAES_set_decrypt_key=CASC_AES_set_decrypt_key")
        .cxxflag("-DAES_encrypt=CASC_AES_encrypt")
        .cxxflag("-DAES_decrypt=CASC_AES_decrypt")
        .cxxflag("-DAES_cbc_decrypt=CASC_AES_cbc_decrypt");

    // Compile CascLib's *bundled* zlib (`src/zlib/*.c`) into `libcasc`, with all
    // zlib symbols renamed via zlib's own `Z_PREFIX` so they cannot collide with
    // the app's other zlib (flate2 -> libz-sys). This is the single change that
    // fixes the launcher's "casc: File not found" regression.
    //
    // Root cause: CascLib's CMakeLists links the host's system zlib whenever
    // `find_package(ZLIB)` succeeds (`-DCASC_USE_SYSTEM_ZLIB`, `ZLIB::ZLIB`).
    // On this build host `find_package` discovers PostgreSQL's `zlib.lib`
    // (exposed via the `LIB` env var, version 1.3.1). That archive is built
    // `/MD`; under the launcher's `+crt-static` (/MT) build, zlib's `inflate`
    // does its internal allocations on the dynamic-CRT heap while CascLib drives
    // it from the static-CRT heap. The cross-CRT state corrupts BLTE
    // decompression and CascLib reports valid files as ERROR_FILE_NOT_FOUND
    // ("casc: File not found") even though the storage and path are correct.
    // This is exactly the "CASC regression under crt-static" symptom.
    //
    // Two naive fixes both fail:
    //   - just disabling find_package -> CascLib compiles its bundled zlib, but
    //     its public symbols (inflate/zutil's z_errmsg/zlibVersion/...) collide
    //     with libz-sys's zlib at the final app link: LNK2005 multiply-defined.
    //   - pointing find_package at libz-sys's archive -> Cargo does not reliably
    //     export `DEP_Z_ROOT` to this build script, so the lookup is unstable.
    //
    // The robust fix: force the bundled-zlib branch AND define `Z_PREFIX`. zlib's
    // `zconf.h` then macro-renames every public symbol to a `z_`-prefixed name
    // (`inflate` -> `z_inflate`, etc.) consistently across the bundled zlib *and*
    // CascDecompress.cpp's call sites. The result is a self-contained `/MT`
    // CascLib whose zlib is private: one CRT (no decompression corruption) and no
    // symbol collision with libz-sys (no LNK2005). Independent of the host's
    // `LIB`/`PATH` and of any other crate's zlib.
    cfg.define("CMAKE_DISABLE_FIND_PACKAGE_ZLIB", "ON");
    // Make the bundled zlib's symbols private so they cannot collide with the
    // app's other zlib (flate2 -> libz-sys) at the final link (LNK2005).
    //
    // `Z_PREFIX` is zlib's own mechanism for exactly this: `zconf.h` macro-renames
    // the entire public + internal API to `z_`-prefixed names (inflate -> z_inflate,
    // inflate_copyright -> z_inflate_copyright, zcalloc -> z_zcalloc, ...), applied
    // consistently to the bundled zlib C sources AND CascDecompress.cpp's call
    // sites (both compiled with this flag). The one symbol it does NOT cover is the
    // `z_errmsg` error-table global (already `z_`-prefixed in vanilla zlib), so we
    // rename that one explicitly. Together these make CascLib's zlib fully private.
    let zdefs = ["-DZ_PREFIX", "-Dz_errmsg=casc_z_errmsg"];
    for d in zdefs {
        cfg.cflag(d);
        cfg.cxxflag(d);
    }

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
