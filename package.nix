{ rustPlatform }:

rustPlatform.buildRustPackage {
  src = ./.;

  name = with builtins; (fromTOML (readFile ./Cargo.toml)).package.name;

  cargoHash = "sha256-2Xfp4w/UB0kpq+Qb6Tqh8r06U7Yeut2CRC7a+7ujLik=";

  meta.mainProgram = "dismus";
}
