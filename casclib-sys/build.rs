extern crate cmake;

use std::env;

fn main() {
    // Gets CacsLib source path from env CASCLIB_DIR
    let casclib_path = env::var("CASCLIB_DIR").unwrap_or("../deps/CascLib".to_string());

    println!("cargo:rerun-if-changed={}", casclib_path);

    let mut cfg = cmake::Config::new(&casclib_path);

    #[cfg(target_os = "windows")]
    {
      cfg.cxxflag("-D UNICODE")
        .cxxflag("-D _UNICODE")
        .cxxflag("-D CASC_UNICODE")
        .cxxflag("-D CASCLIB_UNICODE");        
    
    }

    // Builds CascLib using cmake
    let dst = cfg
        .define("CASC_BUILD_SHARED_LIB", "OFF")
        .define("CASC_BUILD_STATIC_LIB", "ON")
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
