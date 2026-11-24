{
  description = "Rustmius v2 development environment (egui + eframe)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      libs = with pkgs; [
        wayland
        libxkbcommon
        libGL
        libx11
        libxcursor
        libxrandr
        libxi
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          gcc
          pkg-config
        ];

        buildInputs = libs;

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
      };
    };
}
