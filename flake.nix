{
  description = "Umpire: a blazingly fast competitive programming helper";

  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = manifest.name;
            inherit (manifest) version;

            src = pkgs.lib.cleanSource self;
            cargoLock.lockFile = ./Cargo.lock;

            postInstall = ''
              ln -s $out/bin/um $out/bin/umpire
            '';

            meta = {
              inherit (manifest) description;
              homepage = manifest.repository;
              license = pkgs.lib.licenses.gpl3Only;
              mainProgram = "um";
              platforms = pkgs.lib.platforms.unix;
            };
          };
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/um";
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
              gcc
            ];
          };
        }
      );
    };
}
