#!/usr/bin/env bash

# @describe Starts QEMU with a legacy-BIOS-bootable iso image.
#
# @option --iso! Path to bootable image

set -euo pipefail

eval "$(argc --argc-eval "$0" "$@")"

# add "-d int \" to debug CPU exceptions
# "-display none" is necessary for the CI but locally the display and the
#   combat monitor are really helpful

set +e
qemu-system-x86_64 \
    `#-s -S` \
    "-d" "int,cpu_reset" \
    -boot d \
    -cdrom "$argc_iso" \
    -m 32m \
    -debugcon stdio \
    -no-reboot \
    -device isa-debug-exit,iobase=0xf4,iosize=0x04

EXIT_CODE=$?
# Custom exit code used by the integration test to report success.
QEMU_EXIT_SUCCESS=73


echo "#######################################"
if [[ $EXIT_CODE -eq $QEMU_EXIT_SUCCESS ]]; then
    echo "SUCCESS - Integration Test"
    exit 0
else
    echo "FAILED - Integration Test"
    exit "$EXIT_CODE"
fi
