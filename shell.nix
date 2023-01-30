{ pkgs ? import <nixpkgs> {}}:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ rustc cargo rustfmt clippy cargo-watch cargo-edit llvm pkg-config openssl openssl.dev sqlite ];
}
