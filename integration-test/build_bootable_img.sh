#!/usr/bin/env bash

# @describe This script builds a bootable GRUB image for legacy x86 boot. It
#           bundles the PhipsBoot binary along with relevant boot modules into a
#           standalone GRUB installation. GRUB then chainloads PhipsBoot via
#           Multiboot2.
#
# @option --phipsboot! Path to PhipsBoot binary
# @option --out-path! Output directory for the `.iso` file.

set -euo pipefail

eval "$(argc --argc-eval "$0" "$@")"

SELF_DIR=$(dirname "$(realpath "$0")")

GRUB_ISO_VOLUME="${argc_out_path}/.iso-volume"

# Delete previous state.
rm -rf "$GRUB_ISO_VOLUME"

mkdir -p "$GRUB_ISO_VOLUME/boot/grub"
cp "${SELF_DIR}/grub.cfg" "$GRUB_ISO_VOLUME/boot/grub"
# File name must match grub.cfg.
cp "${argc_phipsboot}" "$GRUB_ISO_VOLUME/phipsboot"

# Create a GRUB image with the files in ".vol" being embedded.
grub-mkrescue -o "$argc_out_path/phipsboot.grub-mb2.iso" "$GRUB_ISO_VOLUME" 2>/dev/null
