{
  description = "Rust package using webrtc crate";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"; # or unstable if you prefer
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
          config.allowUnfree = true;

        };

        rust = pkgs.rust-bin.stable.latest.default.override {
          targets = [
           
          ];
        };

      in
      {
        packages.default = rust.buildRustPackage rec {
          pname = "openfab";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            mdbook
            openssl
            mermaid-cli
          ];

          # Some crates use system SSL paths or need environment hints
          RUSTFLAGS = "-C link-arg=-Wl,-rpath,$ORIGIN";

          # WebRTC needs system SSL + crypto headers available
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.openssl
            pkgs.libva
          ];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rust
            mdbook
            mermaid-cli
            pkg-config
            openssl
            nasm
            livekit-libwebrtc
            glib
            libva
          ];

          VULKAN_DIR = "${pkgs.vulkan-loader}";
          WAYLAND_DIR= "${pkgs.wayland}";
          LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib/:${pkgs.vulkan-loader}/lib/:${pkgs.libva.out}/lib/";
          RUST_BACKTRACE = "1";
        };
      });
}