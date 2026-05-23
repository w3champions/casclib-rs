{
  lib,
  rustPlatform,
  cargo-xwin,
  cmake,
  ninja,
  llvmPackages,
  callPackage,
  # Vendored CascLib C++ source tree.
  casclibSrc ? callPackage ./casclib-src.nix { },
  # Pre-built MSVC CRT + Windows SDK cache (the FOD from ./xwin-sdk.nix).
  # Override with a different arch/version by passing your own xwin-sdk
  # invocation.
  xwinSdk ? callPackage ./xwin-sdk.nix { },
  # cargo-xwin's default architecture set is [x86_64, aarch64]. If this
  # doesn't match the arches in the cached xwinSdk's DONE file, cargo-xwin
  # falls back to network download and the sandbox kills the build. Keep
  # this in sync with the xwinSdk arg.
  xwinArch ? "x86_64",
  version ? "0.2.1",
}:

let
  src = ../.;
in

# Linux → Windows MSVC cross-compile. Sandboxed: the SDK comes from the
# xwinSdk FOD, copied into a writable cache so cargo-xwin can populate its
# generated cmake/clang-cl wrapper bits at build time.
rustPlatform.buildRustPackage {
  pname = "casclib-rs-windows";
  inherit version src;

  cargoLock.lockFile = src + "/Cargo.lock";

  nativeBuildInputs = [
    cargo-xwin
    cmake
    # cargo-xwin's generated CMake toolchain file pins the generator to
    # Ninja; without it cmake errors out with "unable to find a build
    # program corresponding to Ninja".
    ninja
    llvmPackages.clang
    llvmPackages.lld
    llvmPackages.llvm
  ];

  env = {
    CASCLIB_DIR = toString casclibSrc;
    CARGO_BUILD_TARGET = "x86_64-pc-windows-msvc";
    XWIN_ARCH = xwinArch;
  };

  preBuild = ''
    # $NIX_BUILD_TOP is the nix build sandbox dir; nothing else writes there
    # during this build, so use it as our xwin cache root.
    export XWIN_CACHE_DIR=$NIX_BUILD_TOP/xwin-cache
    # cargo-xwin appends `/xwin` to XWIN_CACHE_DIR before looking for
    # DONE/crt/sdk, so mirror that layout here.
    mkdir -p $XWIN_CACHE_DIR/xwin
    # cp -a preserves the casing-fix symlinks xwin creates between
    # Windows-style "Include"/"Lib" and POSIX "include"/"lib".
    cp -a ${xwinSdk}/. $XWIN_CACHE_DIR/xwin/
    chmod -R u+w $XWIN_CACHE_DIR
  '';

  # Override the cargo-build hook to invoke cargo-xwin instead. Scope to
  # casclib-sys so we don't drag casclib-bindgen (a dev-only binary that
  # pulls bindgen + clang-sys) into the windows cross build.
  buildPhase = ''
    runHook preBuild
    cargo xwin build --release --target x86_64-pc-windows-msvc -p casclib-sys
    runHook postBuild
  '';

  # We only ship the C static archive. The workspace rlibs are tied to this
  # exact rustc version and not portable, and downstream Rust consumers
  # rebuild from source via cargo anyway.
  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib
    cp target/x86_64-pc-windows-msvc/release/build/casclib-sys-*/out/build/casc.lib $out/lib/
    runHook postInstall
  '';

  dontUseCmakeConfigure = true;
  # Cross-compiled tests would need wine in the sandbox; intentionally
  # build-only here. Run tests via casclib-linux.nix or `cargo xwin test`
  # in the devshell.
  doCheck = false;

  meta = with lib; {
    description = "MSVC-target casc.lib cross-compiled from Linux via cargo-xwin";
    homepage = "https://github.com/ladislav-zezula/CascLib";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
