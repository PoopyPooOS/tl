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
    bun
  ];
  TREE_SITTER_ABI_VERSION = "14";
}
