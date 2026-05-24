{
  description = "casclib-rs — Rust bindings for CascLib (native Linux + Linux→Windows MSVC cross-compile)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # fenix reads rust-toolchain.toml directly via fromToolchainFile, so the
    # nix-built toolchain and any rustup-driven toolchain pick from the same
    # source of truth — no drift between the cross build, the devshell, and
    # a future CI runner.
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      # Pinned by ./rust-toolchain.toml. The sha256 covers the full resolved
      # toolchain bundle (rustc + cargo + rust-std for both targets +
      # rustfmt + clippy); bumping the channel or targets in the toml will
      # invalidate this hash and nix will print the new one.
      rustToolchain = fenix.packages.${system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-gh/xTkxKHL4eiRXzWv8KP7vfjSk61Iq48x47BEDFgfk=";
      };

      # rustPlatform.buildRustPackage uses nixpkgs's rustc by default, which
      # only ships the host target. Both packages go through the pinned
      # fenix toolchain via this rustPlatform so the linux build, the
      # windows cross build, and the devshell all use the same rustc/cargo.
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };

      # callPackage variant that injects our pinned rustPlatform in place of
      # nixpkgs's default, so the package files in ./nix can be written in
      # plain callPackage style without referring to fenix or any flake
      # input.
      callPackage = pkgs.lib.callPackageWith (pkgs // { inherit rustPlatform; });

      casclibSrc = callPackage ./nix/casclib-src.nix { };
      xwinSdk = callPackage ./nix/xwin-sdk.nix { };
      linuxPackage = callPackage ./nix/casclib-linux.nix { inherit casclibSrc; };
      windowsPackage = callPackage ./nix/casclib-windows.nix { inherit casclibSrc xwinSdk; };
      casclibBindgen = callPackage ./nix/casclib-bindgen.nix { inherit xwinSdk; };
    in
    {
      packages.${system} = {
        default = linuxPackage;
        linux = linuxPackage;
        windows = windowsPackage;
        casclib-bindgen = casclibBindgen;
        inherit xwinSdk;
      };

      devShells.${system}.default =
        let
          # build.rs emits `cargo:rustc-link-lib=z` and `=stdc++` (dynamic).
          # rustc doesn't add an rpath entry to the test binary for those,
          # so `cargo test` builds fine but the binary fails to dlopen
          # libz.so.1 / libstdc++.so.6 at startup. Set the loader path
          # explicitly — mirrors the preCheck plumbing in
          # nix/casclib-linux.nix.
          runtimeLibs = [
            pkgs.zlib
            pkgs.stdenv.cc.cc.lib
          ];
          runtimeLibPath = pkgs.lib.makeLibraryPath runtimeLibs;
        in
        pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-xwin
            cmake
            pkg-config
            llvmPackages.clang
            llvmPackages.lld
            llvmPackages.llvm
          ];

          buildInputs = runtimeLibs;

          shellHook = ''
            export XWIN_CACHE_DIR="$HOME/.cache/xwin"
            export LD_LIBRARY_PATH="${runtimeLibPath}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            echo "casclib-rs devshell"
            echo "  native:  cargo build --release -p casclib-sys"
            echo "  windows: cargo xwin build --release --target x86_64-pc-windows-msvc -p casclib-sys"
          '';
        };
    };
}
