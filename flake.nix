{
  description = "Rustmius development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          gcc
          pkg-config
        ];

        buildInputs = with pkgs; [
          openssl
          gtk4
          vte-gtk4
          glib
          pango
          cairo
          gdk-pixbuf
          graphene
        ];
      };
    };
}
