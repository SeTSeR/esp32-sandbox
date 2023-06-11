{
  description = "A shell for my project";

  outputs = { self, nixpkgs }: {
    devShells.aarch64-linux.default =
      let pkgs = nixpkgs.legacyPackages.aarch64-linux;
      in pkgs.mkShell {
        packages = with pkgs; [
          cargo-generate
          cargo-espflash
          rust-analyzer
          rustfmt
          rustup
          gcc13
        ];
        shellHook = ''
export LD_LIBRARY_PATH=${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.zlib}/lib
source ~/export-esp.sh
        '';
    };
  };
}
