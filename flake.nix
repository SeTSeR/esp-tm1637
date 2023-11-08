{
  description = "A shell for my project";

  outputs = { self, nixpkgs }:
  let pkgs = nixpkgs.legacyPackages.aarch64-linux;
  in {
    openocd = pkgs.openocd.overrideAttrs (curr: prev: {
      src = pkgs.fetchFromGitHub {
        owner = "espressif";
        repo = "openocd-esp32";
        rev = "v0.12.0-esp32-20230419";
        sha256 = "sha256-5smeLFiUdVMkuJSOCgSLubB5Bz05adgfEkG5O1lDGzU=";
      };
      nativeBuildInputs = prev.nativeBuildInputs ++ [ pkgs.autoreconfHook ];
      buildInputs = prev.buildInputs ++ [ pkgs.zlib ];
    });

    devShells.aarch64-linux.default = pkgs.mkShell {
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
