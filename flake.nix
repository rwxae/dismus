{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      eachSystem =
        fn:
        nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed (system: fn nixpkgs.legacyPackages.${system});
    in
    {
      packages = eachSystem (pkgs: {
        default = pkgs.callPackage ./package.nix { };
      });

      devShells = eachSystem (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            cargo-expand
            clippy
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      });
    };
}
