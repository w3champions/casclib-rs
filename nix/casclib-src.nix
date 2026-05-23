{ fetchFromGitHub }:

# CascLib C++ source. Pinned to the same revision as the deps/CascLib git
# submodule (07ab5f3). The Rust crates' build.rs reads this path via
# CASCLIB_DIR so the nix build doesn't require the submodule to be
# initialized in the source tree.
fetchFromGitHub {
  owner = "ladislav-zezula";
  repo = "CascLib";
  rev = "07ab5f37ad282cc101d5c17793c550a0a6d4637f";
  hash = "sha256-E1Z4Y1i3KbMuG17M0L3xCLVVcvAGzF5NWWOadAAw3ZQ=";
}
