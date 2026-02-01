#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

EFI_PATH="${1:?missing path to .efi artifact}"
ESP_IMG="${ESP_IMG:-esp.img}"

# Determine mode
# - Auto: if artifact name contains "-tests" or "test"
# - Override: UEFI_RUN_MODE=test|normal
MODE="${UEFI_RUN_MODE:-auto}"
EFI_BASE="$(basename "$EFI_PATH")"

if [[ "$MODE" == "auto" ]]; then
  if [[ "$EFI_BASE" == *"boot-tests"* || "$EFI_BASE" == *"tests"* || "$EFI_BASE" == *"test"* ]]; then
    MODE="test"
  else
    MODE="normal"
  fi
fi

echo "[runner] artifact: $EFI_BASE"
echo "[runner] mode:     $MODE"

if [[ "$MODE" == "test" ]]; then
    KERNEL_SRC="$SCRIPT_DIR/resources/test-kernel.bin" ESP_IMG="$ESP_IMG" BOOT_EFI_SRC="$EFI_PATH" ./scripts/mkesp.sh
  else
    ESP_IMG="$ESP_IMG" BOOT_EFI_SRC="$EFI_PATH" ./scripts/mkesp.sh
fi

# Boot
set +e
ESP_IMG="$ESP_IMG" RUN_MODE="$MODE" ./scripts/run.sh
rc=$?
set -e

# If this is NOT a test run, just return QEMU’s exit code.
if [[ "$MODE" != "test" ]]; then
  echo "[runner] qemu exit code: $rc"
  exit "$rc"
fi

# Test mode: decode isa-debug-exit convention
# Common convention: qemu exits with (value << 1) | 1
# If your Rust writes 0x10 for success and 0x11 for failure:
case "$rc" in
  33)  echo "[runner] tests: PASS"; exit 0 ;;  # (0x10<<1)|1
  35)  echo "[runner] tests: FAIL"; exit 101 ;;  # (0x11<<1)|1
  *)
    echo "[runner] qemu exit code: $rc (not a recognized test exit code)"
    exit 1
    ;;
esac
