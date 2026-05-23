{
  lib,
  rustPlatform,
  makeWrapper,
  llvmPackages,
  callPackage,
  # Pre-built MSVC CRT + Windows SDK cache (the FOD from ./xwin-sdk.nix). The
  # tool reads XWIN_DIR at runtime when invoked with BINDGEN_TARGET_OS=windows
  # to drive clang against the Windows headers.
  xwinSdk ? callPackage ./xwin-sdk.nix { },
  version ? "0.1.0",
}:

let
  src = ../.;
in

# Maintainer tool: regenerates the FFI bindings under casclib-sys/src by
# running bindgen against deps/CascLib/src. The default behavior writes the
# host-target bindings (linux on linux); set BINDGEN_TARGET_OS=windows to
# cross-generate the Windows bindings using the xwinSdk SDK headers.
#
# Example:
#   nix run .#casclib-bindgen            # regen bindings_linux.rs
#   BINDGEN_TARGET_OS=windows \
#   XWIN_DIR=$(nix build .#xwinSdk --print-out-paths) \
#     nix run .#casclib-bindgen          # regen bindings_windows.rs
#
# Bindings are written into the caller's working directory under
# ./casclib-sys/src, so this needs to be run from the repo root.
rustPlatform.buildRustPackage {
  pname = "casclib-bindgen";
  inherit version src;

  cargoLock.lockFile = src + "/Cargo.lock";
  cargoBuildFlags = [
    "-p"
    "casclib-bindgen"
  ];

  # bindgen needs libclang at runtime. Bake the path into the wrapper so the
  # binary works regardless of the caller's env. XWIN_DIR points at the
  # xwinSdk FOD by default; override at invocation time if cross-generating
  # for a different SDK.
  postInstall = ''
    wrapProgram $out/bin/casclib-bindgen \
      --set LIBCLANG_PATH ${llvmPackages.libclang.lib}/lib \
      --set-default XWIN_DIR ${xwinSdk}
  '';

  nativeBuildInputs = [ makeWrapper ];

  doCheck = false;

  meta = with lib; {
    description = "Regenerate casclib-sys FFI bindings via bindgen";
    homepage = "https://github.com/wc3tools/casclib-rs";
    license = licenses.mit;
    mainProgram = "casclib-bindgen";
    platforms = platforms.linux;
  };
}
