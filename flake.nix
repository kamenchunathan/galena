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
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            config.allowUnsupportedSystem = true;
            overlays = [ rust-overlay.overlays.default self.overlays.default ];
          };
          rocPkgs = roc.packages.${system};
          wasilibcDrv = import ./nix/wasilibc.nix
            {

              stdenv = pkgs.stdenv;
              fetchurl = pkgs.fetchurl;
              lib = pkgs.lib;
            };
        in
        f { inherit pkgs; inherit rocPkgs; inherit wasilibcDrv; });
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          rust.fromRustupToolchainFile ./rust-toolchain.toml;
      };

      devShells = forEachSupportedSystem ({ pkgs, rocPkgs, wasilibcDrv }: {
        default = pkgs.mkShell.override
          {
            stdenv = pkgs.clangStdenv;
          }
          {
            packages = with pkgs; [
              # rust
              rustToolchain
              openssl
              pkg-config
              cargo-watch
              rust-analyzer
              wasilibcDrv

              # roc
              rocPkgs.cli
              rocPkgs.lang-server

              # wasm tools
              wasmtime
              wasm-tools
              wabt

              zig
              zls

              llvmPackages_18.libclang
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
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              WASI_LIBC = "${wasilibcDrv}/wasi-sdk-25.0-x86_64-linux/share/wasi-sysroot/lib/wasm32-wasi/libc.a";
              WASI_COMPILER_BUILTINS_RT = "${wasilibcDrv}/wasi-sdk-25.0-x86_64-linux/lib/clang/19/lib/wasi/libclang_rt.builtins-wasm32.a";
            };
          };
      });
    };
}
