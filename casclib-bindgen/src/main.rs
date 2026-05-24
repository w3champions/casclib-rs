use std::env;
use std::path::PathBuf;

fn main() {
    // Target selection. Default to the host OS; override with BINDGEN_TARGET_OS
    // to cross-generate (e.g. produce bindings_windows.rs from a Linux host
    // using xwin's headers).
    let target_os = env::var("BINDGEN_TARGET_OS").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else {
            "linux".to_string()
        }
    });

    let mut builder = bindgen::Builder::default()
        .clang_arg("-I./deps/CascLib/src")
        .header("./casclib-bindgen/src/wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("^CASC.+")
        .allowlist_function("^Casc.+")
        .allowlist_function("SetCascError")
        .allowlist_function("GetCascError")
        .allowlist_var("^CASC.+")
        .allowlist_var("^ERROR_.+")
        .allowlist_var("FILE_BEGIN")
        .allowlist_var("FILE_CURRENT")
        .allowlist_var("FILE_END");

    // When cross-generating Windows bindings from a non-Windows host, point
    // clang at the MSVC CRT + Windows SDK headers via XWIN_DIR (the xwinSdk
    // FOD's output path). CascPort.h then sees _WIN32 and pulls in
    // <windows.h>, producing the Windows-flavored type layout.
    if target_os == "windows" && !cfg!(target_os = "windows") {
        let xwin_dir =
            env::var("XWIN_DIR").expect("XWIN_DIR must be set to cross-generate windows bindings");
        builder = builder
            .clang_arg("--target=x86_64-pc-windows-msvc")
            .clang_arg(format!("-I{xwin_dir}/crt/include"))
            .clang_arg(format!("-I{xwin_dir}/sdk/include/ucrt"))
            .clang_arg(format!("-I{xwin_dir}/sdk/include/um"))
            .clang_arg(format!("-I{xwin_dir}/sdk/include/shared"))
            .clang_arg(format!("-I{xwin_dir}/sdk/include/winrt"));
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let file_name = match target_os.as_str() {
        "windows" => "bindings_windows.rs",
        "macos" => "bindings_macos.rs",
        _ => "bindings_linux.rs",
    };
    let out_path = PathBuf::from("./casclib-sys/src").join(file_name);
    bindings
        .write_to_file(&out_path)
        .expect("Couldn't write bindings!");
    println!("wrote {}", out_path.display());
}
