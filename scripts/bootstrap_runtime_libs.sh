#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME_DIR="$ROOT_DIR/.runtime"
PKG_DIR="$RUNTIME_DIR/pkgs"
EXTRACT_DIR="$RUNTIME_DIR/extract"
LIB_DIR="$RUNTIME_DIR/lib"

REQUIRED_LIBS=(
  "libfmt.so.9"
  "libspdlog.so.1.12"
  "liblttng-ust.so.1"
)

REQUIRED_PACKAGES=(
  "libfmt9"
  "libspdlog1.12"
  "liblttng-ust1t64"
  "liblttng-ust-common1t64"
  "liblttng-ust-ctl5t64"
)

all_libs_present() {
  local lib
  for lib in "${REQUIRED_LIBS[@]}"; do
    if [[ ! -e "$LIB_DIR/$lib" ]]; then
      return 1
    fi
  done
  return 0
}

if all_libs_present; then
  exit 0
fi

command -v apt >/dev/null 2>&1 || {
  echo "Missing required command: apt" >&2
  exit 1
}

command -v dpkg-deb >/dev/null 2>&1 || {
  echo "Missing required command: dpkg-deb" >&2
  exit 1
}

mkdir -p "$PKG_DIR" "$EXTRACT_DIR" "$LIB_DIR"

(
  cd "$PKG_DIR"
  apt download "${REQUIRED_PACKAGES[@]}"
)

for deb in "$PKG_DIR"/*.deb; do
  deb_name="$(basename "$deb" .deb)"
  deb_extract_dir="$EXTRACT_DIR/$deb_name"
  mkdir -p "$deb_extract_dir"
  dpkg-deb -x "$deb" "$deb_extract_dir"
done

find "$EXTRACT_DIR" -path '*/usr/lib/x86_64-linux-gnu/*' \( -type f -o -type l \) \
  -exec cp -a -t "$LIB_DIR" {} +

all_libs_present || {
  echo "Failed to provision required runtime libraries into $LIB_DIR" >&2
  exit 1
}
