{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, rust-overlay }: {
    devShells.x86_64-linux.default =
      with import nixpkgs {
        system = "x86_64-linux";
        overlays = [ rust-overlay.overlays.default ];
      };
      mkShellNoCC {
        hardeningDisable = [ "all" ];

        CC_riscv64gc_unknown_linux_gnu = "${pkgsCross.riscv64.stdenv.cc.targetPrefix}gcc";

        nativeBuildInputs = [
          pkgsCross.riscv64.stdenv.cc
        ];

        buildInputs = [
          cargo-asm cargo-watch cargo-binutils rust-analyzer
          (rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
              extensions = [ "rust-src" "llvm-tools-preview" ];
              targets = [ "x86_64-unknown-linux-gnu" "riscv64gc-unknown-none-elf" "riscv64gc-unknown-linux-gnu" ];
            }))
        ];
      };
  };
}
