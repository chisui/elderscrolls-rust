{ pkgs ? import <nixpkgs> {} }:
with pkgs;
mkShell {
  name = "bsa";
  buildInputs = [
    rustup
  ];
}
