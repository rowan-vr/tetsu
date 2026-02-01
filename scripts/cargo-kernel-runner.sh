#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "$REPO_ROOT"

EFI_PATH="${EFI_PATH:-build}" # build if not found
ESP_IMG="${ESP_IMG:-esp.img}"

KERNEL_ELF="${1:?missing path to kernel binary}"
KERNEL_BIN="${KERNEL_BIN:-build/tetsu-kernel.bin}"

# Determine mode
# - Auto: if artifact name contains "-tests" or "test"
# - Override: RUN_MODE=test|normal
MODE="${RUN_MODE:-auto}"
KERNEL_BASE="$(basename "$KERNEL_ELF")"

if [[ "$MODE" == "auto" ]]; then
  if [[ "$KERNEL_BASE" == *"test"* ]]; then
    MODE="test"
  else
    MODE="normal"
  fi
fi

echo "[runner] artifact: $KERNEL_BASE"
echo "[runner] mode:     $MODE"

if [[ "$MODE" == "test" ]]; then
  EFI_PATH="$SCRIPT_DIR/resources/test-boot.efi"
fi

if [[ "$EFI_PATH" == "build" ]]; then
  echo "[runner] building uefi image..."
  cargo +nightly build -p tetsu-boot --target x86_64-unknown-uefi
  EFI_PATH="$REPO_ROOT/target/x86_64-unknown-uefi/debug/tetsu-boot.efi"
fi

echo "[runner] Convert ELF -> flat binary ($KERNEL_BIN)"
objcopy -O binary "$KERNEL_ELF" "$KERNEL_BIN"
head4="$(xxd -l 4 -p "$KERNEL_BIN" 2>/dev/null || true)"
if [[ "$head4" == "00000000" ]]; then
  echo "WARNING: kernel.bin begins with 00000000." >&2
  echo "If your linker places .text at 0x00100000, and you're using a raw loader," >&2
  echo "make sure you're copying the right artifact and that _start is at 0x00100000." >&2
fi

if [[ "$MODE" == "test" ]]; then
    KERNEL_SRC="$KERNEL_BIN" ESP_IMG="$ESP_IMG" BOOT_EFI_SRC="$EFI_PATH" ./scripts/mkesp.sh
  else
    KERNEL_SRC="$KERNEL_BIN" ESP_IMG="$ESP_IMG" BOOT_EFI_SRC="$EFI_PATH" ./scripts/mkesp.sh
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
