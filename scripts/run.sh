#!/usr/bin/env bash
set -euo pipefail

# run.sh — Boot TetsuOS in QEMU (UEFI / OVMF)
#
# Usage:
#   ./scripts/run.sh
#   RAM=1024 ./scripts/run.sh
#   ESP_IMG=build/esp.img ./scripts/run.sh
#   ESP_IMG=build/esp.img RUN_MODE=test ./scripts/run.sh
#
# Optional env vars:
#   ESP_IMG       (default: esp.img)
#   RAM           (default: 512)
#   OVMF_CODE     (auto-detected if not set)
#   OVMF_VARS     (default: OVMF_VARS.fd)
#   RUN_MODE      (default: normal)

ESP_IMG="${ESP_IMG:-esp.img}"
RAM="${RAM:-512}"
OVMF_VARS="${OVMF_VARS:-OVMF_VARS.fd}"
MODE="${RUN_MODE:-normal}"
RUN_HEADLESS="${RUN_HEADLESS:-0}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "Error: missing required command: $1" >&2
    exit 1
  }
}

need_cmd qemu-system-x86_64

if [[ ! -f "$ESP_IMG" ]]; then
  echo "Error: ESP image not found: $ESP_IMG"
  echo "Run ./scripts/mkesp.sh first."
  exit 1
fi

# Auto-detect OVMF_CODE if not set
if [[ -z "${OVMF_CODE:-}" ]]; then
  for path in \
    /usr/share/OVMF/OVMF_CODE.fd \
    /usr/share/OVMF/OVMF_CODE.4m.fd \
    /usr/share/OVMF/OVMF_CODE_4M.fd \
    /usr/share/edk2-ovmf/x64/OVMF_CODE.fd \
    /usr/share/edk2-ovmf/x64/OVMF_CODE.4m.fd \
    /usr/share/edk2/x64/OVMF_CODE.4m.fd
  do
    if [[ -f "$path" ]]; then
      OVMF_CODE="$path"
      break
    fi
  done
fi

if [[ -z "${OVMF_CODE:-}" || ! -f "$OVMF_CODE" ]]; then
  echo "Error: Could not find OVMF_CODE.fd"
  echo "Install edk2-ovmf or set OVMF_CODE manually."
  exit 1
fi

# Prepare writable vars file

# Auto-detect OVMF_VARS if not set
if [[ ! -f "$OVMF_VARS" ]]; then

  # Find OVMF_VARS from the system
  if [[ -z "${SYS_OVMF_VARS:-}" ]]; then
    for path in \
      /usr/share/OVMF/OVMF_VARS.fd \
      /usr/share/OVMF/OVMF_VARS.4m.fd \
      /usr/share/OVMF/OVMF_VARS_4M.fd \
      /usr/share/edk2-ovmf/x64/OVMF_VARS.fd \
      /usr/share/edk2-ovmf/x64/OVMF_VARS.4m.fd \
      /usr/share/edk2/x64/OVMF_VARS.4m.fd
    do
      if [[ -f "$path" ]]; then
        SYS_OVMF_VARS="$path"
        break
      fi
    done
  fi

  if [[ -z "${SYS_OVMF_VARS:-}" || ! -f "$SYS_OVMF_VARS" ]]; then
    echo "Error: Could not find OVMF_VARS.fd"
    echo "Install edk2-ovmf or set OVMF_VARS manually."
    exit 1
  fi

  cp $SYS_OVMF_VARS $OVMF_VARS
fi

QEMU_ARGS=(
  -machine q35
  -m "${RAM}M"
  -drive if=pflash,format=raw,readonly=on,file="$OVMF_CODE"
  -drive if=pflash,format=raw,file="$OVMF_VARS"
  -drive format=raw,file="$ESP_IMG"
  -serial stdio
)

if [[  "$RUN_HEADLESS" == "1"]]; then
  QEMU_ARGS+=(-display none)
fi

if [[ "$RUN_MODE" == "test" ]]; then
  echo "[run] Test mode enabled (isa-debug-exit)"
  QEMU_ARGS+=(
    -device isa-debug-exit,iobase=0xf4,iosize=0x04
  )
fi

echo "[run] Booting TetsuOS"
echo "      ESP:  $ESP_IMG"
echo "      RAM:  ${RAM}M"
echo "      OVMF: $OVMF_CODE"
echo "      MODE: $MODE"


set +e
qemu-system-x86_64 "${QEMU_ARGS[@]}"
rc=$?
set -e

echo "[run] qemu exited with code: $rc"
exit "$rc"