#!/usr/bin/env bash
set -euo pipefail

# ===== Config (adjust if your names differ) =====
KERNEL_PKG="${KERNEL_PKG:-tetsu-kernel}"
KERNEL_CUSTOM_TARGET=x86_64-tetsu-kernel
KERNEL_TARGET="${KERNEL_TARGET:-$KERNEL_PKG/$KERNEL_CUSTOM_TARGET.json}"   # or your custom target name
KERNEL_ELF="${KERNEL_ELF:-target/$KERNEL_CUSTOM_TARGET/debug/${KERNEL_PKG}}"
KERNEL_BIN="${KERNEL_BIN:-build/tetsu-kernel.bin}"

ESP_IMG="${ESP_IMG:-esp.img}"                          # optional copy step
COPY_TO_ESP="${COPY_TO_ESP:-1}"                        # set to 0 to skip

# Your existing cargo alias/command that builds+boots UEFI
UEFI_CMD="${UEFI_CMD:-cargo uefi}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Missing command: $1" >&2; exit 1; }
}

need_cmd cargo
need_cmd objcopy
mkdir -p "$(dirname "$KERNEL_BIN")"

echo "[1/3] Build kernel ($KERNEL_PKG) for $KERNEL_TARGET"
# Use nightly + build-std so no_std kernels work consistently
cargo +nightly build -p "$KERNEL_PKG" \
  -Z build-std=core \
  -Z build-std-features=compiler-builtins-mem \
  --target "$KERNEL_TARGET"

if [[ ! -f "$KERNEL_ELF" ]]; then
  echo "Kernel ELF not found at: $KERNEL_ELF" >&2
  echo "Tip: run with KERNEL_TARGET=... or set KERNEL_ELF=..." >&2
  exit 1
fi

echo "[2/3] Convert ELF -> flat binary ($KERNEL_BIN)"
objcopy -O binary "$KERNEL_ELF" "$KERNEL_BIN"

# Sanity check (catches the “wrong target folder” problem)
head4="$(xxd -l 4 -p "$KERNEL_BIN" 2>/dev/null || true)"
echo "[2/3] kernel.bin head: ${head4:-<no xxd>}"
if [[ "$head4" == "00000000" ]]; then
  echo "WARNING: kernel.bin begins with 00000000." >&2
  echo "If your linker places .text at 0x00100000, and you're using a raw loader," >&2
  echo "make sure you're copying the right artifact and that _start is at 0x00100000." >&2
fi


echo "[3/3] Build + run UEFI ($UEFI_CMD)"
exec $UEFI_CMD
