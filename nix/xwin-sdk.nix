{
  stdenvNoCC,
  xwin,
  cacert,
  # Architecture passed to `xwin splat --arch`. Drives both the SDK contents
  # and the `DONE` marker that cargo-xwin reads to skip its download path.
  # Must agree with XWIN_ARCH in any cargo-xwin invocation that consumes the
  # resulting cache (see casclib-windows.nix).
  xwinArch ? "x86_64",
  # SDK major version (matches XWIN_VERSION / cargo-xwin's --xwin-version
  # default).
  xwinVersion ? "17",
  # Fixed-output hash. xwin's splat result is deterministic for a given arch
  # and SDK version, so this only needs to change when xwinArch or
  # xwinVersion change.
  outputHash ? "sha256-mlF5xa8zjaGD3lYwiO7BR+REHf//qo+APJoyY9/kK3Y=",
}:

# Microsoft MSVC CRT + Windows SDK, fetched once via xwin and cached as a
# fixed-output derivation. Output layout matches what cargo-xwin expects
# under $XWIN_CACHE_DIR/xwin/: { crt/, sdk/, DONE }.
#
# $NIX_BUILD_TOP is set by Nix to the per-build sandbox scratch dir — that's
# where unpacking and intermediate work happens.
stdenvNoCC.mkDerivation {
  pname = "msvc-xwin-sdk";
  version = xwinVersion;

  dontUnpack = true;

  nativeBuildInputs = [
    xwin
    cacert
  ];

  buildPhase = ''
    export HOME=$NIX_BUILD_TOP
    # Splat into the build dir so xwin's internal rename(2) calls stay on a
    # single filesystem (unpack/ and the splat output cannot straddle the
    # build dir and /nix/store — EXDEV).
    mkdir -p $NIX_BUILD_TOP/staging
    xwin --accept-license --arch ${xwinArch} \
         --cache-dir $NIX_BUILD_TOP/xwin-cache \
         splat --output $NIX_BUILD_TOP/staging
    mkdir -p $out
    cp -a $NIX_BUILD_TOP/staging/. $out/
    # cargo-xwin's marker file — its setup_msvc_crt() reads this as a
    # space-separated arch list and skips the download path iff every
    # requested arch is present.
    echo -n "${xwinArch}" > $out/DONE
  '';

  installPhase = "true";

  outputHashMode = "recursive";
  outputHashAlgo = "sha256";
  inherit outputHash;
}
