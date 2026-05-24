# casclib-rs
Safe [CascLib](https://github.com/ladislav-zezula/CascLib) bindings for Rust

```rust
use casclib::{open};

let storage = open(r#"C:\Program Files (x86)\StarCraft II\SC2Data"#).unwrap();

// walk all files
for r in storage.files() {
  let entry = r.expect("file entry");
  let name = entry.get_name();
  if name.ends_with(".galaxy") {
    // found a galaxy file!
  }
}

// extract a file by path
let file = storage.entry(TEST_FILE).open().unwrap();
file.extract(writer).unwrap();
```

## Building

### With cargo (any platform)

```sh
git submodule update --init --recursive   # vendored CascLib in ./deps
cargo build --release -p casclib-sys
```

The `cmake` build script under `casclib-sys/build.rs` drives an in-tree
CMake build of `deps/CascLib` and links the resulting static archive.
You need `cmake`, a C/C++ toolchain, and system `zlib` headers on PATH.
On NixOS, prefer the flake.

### With Nix

The flake provides reproducible builds for native Linux and the
Linux → Windows MSVC cross. The Rust toolchain is pinned via
[`rust-toolchain.toml`](./rust-toolchain.toml) and resolved through
[fenix](https://github.com/nix-community/fenix), so the devshell, the
sandboxed nix builds, and any future rustup user pick up the same
`rustc`/`cargo`.

```sh
# Native Linux: ships ./result/lib/libcasc.a
nix build .#linux        # or just `nix build`

# Linux → Windows MSVC cross-compile: ships ./result/lib/casc.lib
nix build .#windows
```

The linux build runs the workspace check phase (bindgen layout assertions
are compile-time `const _:` checks, integration tests under
`*/tests/casclib.rs` are gated to local dev machines with a real game
install). The Windows build is build-only — running cross-compiled tests
would require `wine` in the sandbox.

#### Devshell

```sh
nix develop
# inside the devshell:
cargo build   --release -p casclib-sys
cargo clippy  --workspace --exclude casclib-bindgen --all-targets
# Windows cross via cargo-xwin (uses ~/.cache/xwin on first run):
cargo xwin build  --release --target x86_64-pc-windows-msvc -p casclib-sys
cargo xwin clippy --workspace --exclude casclib-bindgen \
                  --all-targets --target x86_64-pc-windows-msvc
```

### How the Windows cross-compile works

The flake builds the MSVC CRT + Windows SDK once as a fixed-output
derivation via [`xwin`](https://github.com/Jake-Shadle/xwin) (see the
`xwinSdk` derivation) and feeds the splatted output into
[`cargo-xwin`](https://github.com/rust-cross/cargo-xwin) at build time.
The sandboxed build itself stays fully offline.

The cross target is pinned to `x86_64-pc-windows-msvc`. The `XWIN_ARCH`
env in the windows build derivation must match the `--arch` passed to
the xwinSdk FOD; both are driven from the single `xwinArch` arg in
`nix/casclib-windows.nix` and `nix/xwin-sdk.nix`.

### Regenerating FFI bindings

The bindgen tool is packaged as `.#casclib-bindgen` so libclang and (for
the cross-generated Windows bindings) the Windows SDK headers come from
the same pinned `nixpkgs` and `xwinSdk` as the rest of the build.

```sh
# Regenerate ./casclib-sys/src/bindings_linux.rs against the current
# deps/CascLib headers using the host's clang:
nix run .#casclib-bindgen

# Cross-generate ./casclib-sys/src/bindings_windows.rs from Linux using
# xwin's Windows SDK headers (no Windows machine required):
BINDGEN_TARGET_OS=windows nix run .#casclib-bindgen
```

`bindings_macos.rs` still needs to be regenerated on macOS — bindgen
walks the platform's actual SDK headers, and Apple's SDK isn't a
free-redistributable bundle.

### Bumping the vendored CascLib

CascLib is fetched in the flake as a `fetchFromGitHub` FOD pinned by
commit + hash, and locally as a git submodule under `./deps/CascLib`.
To bump:

1. Update the submodule (`git -C deps/CascLib fetch && git -C deps/CascLib checkout <rev>`).
2. Update `rev` in `nix/casclib-src.nix` to match, set `hash` to
   `lib.fakeHash`, and rebuild to get the real hash.
3. If CascLib's public headers changed, regenerate the FFI bindings
   (see above).

### Bumping the Rust toolchain

Edit `rust-toolchain.toml`, then rebuild any nix package; fenix will
fail with a `sha256` mismatch and print the new hash to paste into
`rustToolchain` in `flake.nix`.
