{
  lib,
  rustPlatform,
  cmake,
  zlib,
  stdenv,
  callPackage,
  # Vendored CascLib C++ source tree. Override to point at a different
  # revision or a local checkout.
  casclibSrc ? callPackage ./casclib-src.nix { },
  version ? "0.3.0",
}:

let
  src = ../.;
in

# Native Linux: rustPlatform.buildRustPackage drives a fully sandboxed,
# offline cargo build against the committed Cargo.lock. The build is scoped
# to casclib-sys so cargo doesn't pull casclib-bindgen (a dev-only tool that
# drags bindgen + clang-sys into the build). The only artifact shipped is
# libcasc.a from the cmake subbuild, mirroring what casclib-windows.nix
# does with casc.lib.
rustPlatform.buildRustPackage {
  pname = "casclib-rs";
  inherit version src;

  cargoLock.lockFile = src + "/Cargo.lock";

  cargoBuildFlags = [
    "-p"
    "casclib-sys"
  ];

  # The integration tests under */tests/casclib.rs open
  # `C:\Program Files (x86)\Warcraft III\Data` directly — they're marked
  # #[ignore] and need `cargo test -- --ignored` on a developer machine
  # with the game installed. The check phase here only runs lib tests,
  # which after bindgen 0.72's switch to compile-time `const _:` layout
  # assertions are validated at build time.
  cargoTestFlags = [
    "--workspace"
    "--exclude"
    "casclib-bindgen"
  ];

  # rustc doesn't rpath buildInputs into the test binaries, so every
  # dynamically-linked C dep (libstdc++, libz) is missing at test-execution
  # time. Point the loader at them for the check phase only — the installed
  # artifact (libcasc.a) is static and unaffected.
  preCheck =
    let
      libraries = lib.makeLibraryPath [
        stdenv.cc.cc.lib
        zlib
      ];
    in
    ''
      export LD_LIBRARY_PATH="${libraries}"
    '';

  nativeBuildInputs = [ cmake ];
  buildInputs = [
    zlib
    # build.rs emits `cargo:rustc-link-lib=stdc++` (dynamic). Without this,
    # the test binary builds fine but the sandbox loader can't find
    # libstdc++.so.6 at test-execution time.
    stdenv.cc.cc.lib
  ];

  env = {
    CASCLIB_DIR = toString casclibSrc;
  };

  # cmake here is invoked by casclib-sys/build.rs via the cmake crate; the
  # package itself isn't a CMake project, so suppress the default configure
  # hook.
  dontUseCmakeConfigure = true;

  # Skip the default install hook (which would copy binaries from
  # target/release/ into $out/bin/) and ship just the static library. Rust
  # consumers cargo-build from source anyway; the rlibs are tied to this
  # exact rustc version and aren't redistributable.
  installPhase = ''
    runHook preInstall
    mkdir -p $out/lib
    # nixpkgs's rustPlatform passes an explicit --target, so cargo writes
    # under target/<triple>/release/ rather than target/release/. The cmake
    # crate places its installed output at .../out/lib/.
    cp target/*/release/build/casclib-sys-*/out/lib/libcasc.a $out/lib/
    runHook postInstall
  '';

  meta = with lib; {
    description = "Static CascLib (libcasc.a) built against the workspace's vendored source";
    homepage = "https://github.com/ladislav-zezula/CascLib";
    license = licenses.mit;
    platforms = platforms.linux;
  };
}
