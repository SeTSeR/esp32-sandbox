{
  description = "A shell for my project";

  outputs = { self, np }: {
    devShells.aarch64-linux.default =
      let pkgs = np.legacyPackages.aarch64-linux;
      in pkgs.mkShell {
        packages = [ pkgs.cargo pkgs.cargo-espflash pkgs.cargo-generate pkgs.rustup pkgs.gcc13 ];
    };
  };
}
