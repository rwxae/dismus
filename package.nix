{ rustPlatform }:

rustPlatform.buildRustPackage {
  src = ./.;

  name = with builtins; (fromTOML (readFile ./Cargo.toml)).package.name;

  cargoHash = "sha256-Ydtjrm/d9j7x633pZECWp1gawHY6VsdIStB4Q3uBvMM=";

  meta.mainProgram = "dismus";
}
