#!/usr/bin/env bash
set -euo pipefail

# mkesp.sh — create/format a FAT32 EFI System Partition image for TetsuOS
#
# Usage:
#   ./scripts/mkesp.sh
#   ESP_SIZE_MB=128 ./scripts/mkesp.sh
#   ESP_IMG=build/esp.img ./scripts/mkesp.sh
#
# What it does:
# - (re)creates esp.img as FAT32
# - ensures /EFI/BOOT exists inside the image
# - optionally copies BOOTX64.EFI if it exists at the expected build path

ESP_IMG="${ESP_IMG:-esp.img}"
ESP_SIZE_MB="${ESP_SIZE_MB:-64}"

BOOT_EFI_SRC="${BOOT_EFI_SRC:-target/x86_64-unknown-uefi/debug/tetsu-boot.efi}"
BOOT_EFI_DST="::EFI/BOOT/BOOTX64.EFI"

# Optional: also copy a kernel file if you want (can be empty for now)
KERNEL_SRC="${KERNEL_SRC:-}"
KERNEL_DST="${KERNEL_DST:-::kernel.bin}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Error: missing required command: $1" >&2
    exit 1
  }
}

need_cmd truncate
need_cmd mkfs.fat
need_cmd mmd
need_cmd mcopy
need_cmd mdir

mkdir -p "$(dirname "$ESP_IMG")" 2>/dev/null || true

echo "[mkesp] Creating FAT32 image: $ESP_IMG (${ESP_SIZE_MB}MB)"
rm -f "$ESP_IMG"
truncate -s "${ESP_SIZE_MB}M" "$ESP_IMG"
mkfs.fat -F 32 "$ESP_IMG" >/dev/null

echo "[mkesp] Creating EFI directories"
# mtools paths do NOT need leading slash; also no '::/EFI', use '::EFI'
mmd -i "$ESP_IMG" ::EFI >/dev/null 2>&1 || true
mmd -i "$ESP_IMG" ::EFI/BOOT >/dev/null 2>&1 || true

if [[ -f "$BOOT_EFI_SRC" ]]; then
  echo "[mkesp] Copying bootloader:"
  echo "        $BOOT_EFI_SRC -> $BOOT_EFI_DST"
  mcopy -o -i "$ESP_IMG" "$BOOT_EFI_SRC" "$BOOT_EFI_DST"
else
  echo "[mkesp] Bootloader not found at: $BOOT_EFI_SRC"
  echo "        Build it with:"
  echo "          cargo build -p tetsu-boot --target x86_64-unknown-uefi"
fi

if [[ -n "${KERNEL_SRC}" ]]; then
  if [[ -f "$KERNEL_SRC" ]]; then
    echo "[mkesp] Copying kernel:"
    echo "        $KERNEL_SRC -> $KERNEL_DST"
    mcopy -o -i "$ESP_IMG" "$KERNEL_SRC" "$KERNEL_DST"
  else
    echo "[mkesp] Kernel path set but file missing: $KERNEL_SRC" >&2
    exit 1
  fi
fi

echo "[mkesp] Contents:"
mdir -i "$ESP_IMG" ::EFI/BOOT || true
echo "[mkesp] Done."
