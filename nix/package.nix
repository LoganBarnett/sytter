{ lib
, stdenv
, rustPlatform
, pkg-config
, openssl
, darwin ? {}
}:

rustPlatform.buildRustPackage {
  pname = "sytter";
  # Pin however you like; using a datestamped "unstable" is fine inside a repo.
  version = "unstable-2025-09-08";

  # Use the repository root as the source when packaging in-repo.
  src = lib.cleanSource (../.);

  # Use Cargo.lock from the repo so we don’t need a vendor hash.
  # If you have git-based crates, add outputHashes here as needed.
  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes = { };
  };

  # Native build-time deps (used by build scripts / pkg-config detection).
  nativeBuildInputs = [ pkg-config ];

  # Link-time / runtime deps.
  buildInputs =
    [ openssl ]
    ++ lib.optionals stdenv.isDarwin [
      darwin.apple_sdk.frameworks.CoreFoundation
      darwin.apple_sdk.frameworks.Foundation
      darwin.apple_sdk.frameworks.IOKit
      darwin.apple_sdk.frameworks.SystemConfiguration
    ];

  # If you end up using bindgen or cc, set:
  # RUSTFLAGS or add clang/llvm to nativeBuildInputs.

  # Install examples for easy reference (optional).
  postInstall = ''
    mkdir -p $out/share/sytter/examples
    cp -r examples/* $out/share/sytter/examples/ 2>/dev/null || true
  '';

  meta = {
    description = "Event-driven system babysitter / automation runner";
    homepage = "https://github.com/LoganBarnett/sytter";
    license = lib.licenses.mit;
    mainProgram = "sytter";
    platforms = lib.platforms.unix;
  };
}
