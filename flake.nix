{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    roc.url = "github:roc-lang/roc";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, roc, flake-utils }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default self.overlays.default ];
        };
        rocPkgs = roc.packages.${system};
      });
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          rust.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      devShells = forEachSupportedSystem ({ pkgs, rocPkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            # rust
            rustToolchain
            openssl
            pkg-config
            cargo-watch
            rust-analyzer

            # roc
            rocPkgs.cli
            rocPkgs.lang-server

            # wasm tools
            wasmtime
            wasm-tools
            wabt

            zig
            zls

            llvmPackages_18.libllvm
            llvmPackages_18.bintools-unwrapped
            lldb_18

            vscode-extensions.vadimcn.vscode-lldb

            # command runner
            just

            # node
            nodejs_24
            nodePackages.pnpm
            typescript
          ];


          env = {
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      });
    };
}




