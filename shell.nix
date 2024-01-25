let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs { };
in
pkgs.mkShell rec {
  # CLI Utilities
  nativeBuildInputs = with pkgs; [
    binutils # as and ld
    gcc
    grub2
    gnumake
    qemu
    rustup
    xorriso

    cargo-nextest
  ];
}
