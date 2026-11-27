{
  description = "Rustmius - Fast, native, open-source SSH client and host manager built in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          libs = with pkgs; [
            wayland
            libxkbcommon
            libGL
            libx11
            libxcursor
            libxrandr
            libxi
            vulkan-loader
          ];
          rustmius = pkgs.rustPlatform.buildRustPackage {
            pname = "rustmius";
            version = "3.1.0";
            src = self;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
            ];

            buildInputs = libs ++ (with pkgs; [ openssl aws-lc ]);

            postInstall = ''
              wrapProgram $out/bin/rustmius \
                --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath libs}
            '';

            meta = with pkgs.lib; {
              description = "Fast, native, open-source SSH client and host manager built in Rust";
              homepage = "https://github.com/Cleboost/Rustmius";
              license = licenses.agpl3Plus;
              mainProgram = "rustmius";
              platforms = supportedSystems;
            };
          };
        in
        {
          default = rustmius;
          inherit rustmius;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/rustmius";
        };
      });

      overlays.default = final: prev: {
        rustmius = self.packages.${final.stdenv.hostPlatform.system}.default;
      };

      devShells = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          libs = with pkgs; [
            wayland
            libxkbcommon
            libGL
            libx11
            libxcursor
            libxrandr
            libxi
            vulkan-loader
          ];
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              gcc
              pkg-config
            ];

            buildInputs = libs ++ (with pkgs; [ openssl aws-lc ]);

            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
          };
        }
      );
    };
}
