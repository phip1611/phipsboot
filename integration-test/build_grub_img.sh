#!/usr/bin/env bash

# This script builds a bootable GRUB image for legacy x86 boot. It bundles the
# chainloader binary along with relevant boot modules into a GRUB installation.
# GRUB's config instructs GRUB to chainload the binary via Multiboot2.

# http://redsymbol.net/articles/unofficial-bash-strict-mode/
set -euo pipefail
IFS=$'\n\t'

DIR=$(dirname "$(realpath "$0")")
cd "$DIR" || exit

echo "Verifying that the binary is a multiboot2 binary..."
grub-file --is-x86-multiboot2 "${CHAINLOADER_ARTIFACT}" # env var

# Delete previous state.
rm -rf .vol

mkdir -p .vol/boot/grub
cp grub.cfg .vol/boot/grub
cp "${CHAINLOADER_ARTIFACT}" .vol

# Create a GRUB image with the files in ".vol" being embedded.
grub-mkrescue -o "grub_boot.img" ".vol" 2>/dev/null
