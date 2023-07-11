{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.rustc
    pkgs.cargo
  ] ++ [ pkgs.libiconv pkgs.darwin.apple_sdk.frameworks.CoreServices ];
}
