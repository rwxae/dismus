{ rustPlatform }:

rustPlatform.buildRustPackage {
  src = ./.;

  name = with builtins; (fromTOML (readFile ./Cargo.toml)).package.name;

  cargoHash = "sha256-qe8vX/AaDRs50nrpAyq8ZAIig6lonTVsMFrNK2h5K04=";

  meta.mainProgram = "dismus";
}
