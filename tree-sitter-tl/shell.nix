{
  pkgs ? import <nixpkgs> { },
}:

with pkgs;
mkShell {
  buildInputs = [
    tree-sitter
    nodejs
    gcc
    python3
  ];
  TREE_SITTER_ABI_VERSION = "14";
}
