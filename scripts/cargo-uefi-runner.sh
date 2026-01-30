#!/usr/bin/env bash
set -euo pipefail

# Cargo calls the runner as: <runner> <path-to-built-artifact>
EFI_PATH="${1:?missing path to .efi artifact}"

# Build ESP and copy the exact artifact Cargo just built
ESP_IMG="${ESP_IMG:-esp.img}"

# Ensure ESP exists and has directories
ESP_IMG="$ESP_IMG" BOOT_EFI_SRC="$EFI_PATH" ./scripts/mkesp.sh

# Boot
ESP_IMG="$ESP_IMG" ./scripts/run.sh
