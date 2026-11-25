#!/usr/bin/env bash
# Build Glyphra for the OS you are on. Options are passed to the platform script:
#   ./build.sh            macOS: .app + .dmg      Linux: .tar.gz (+ .deb/.rpm/AppImage)
#   ./build.sh --run      build and start
#   ./build.sh --help     options of the platform script
# Windows: .\scripts\build-windows.ps1  (PowerShell; also reached from Git Bash by this script)
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
case "$(uname -s)" in
  Darwin) exec ./scripts/build-macos.sh "$@" ;;
  Linux) exec ./scripts/build-linux.sh "$@" ;;
  MINGW*|MSYS*|CYGWIN*)
    # PowerShell uses -Run, -Msi, -Arch; map the common bash-style flags.
    args=()
    for a in "$@"; do
      case "$a" in
        --run) args+=(-Run) ;; --msi) args+=(-Msi) ;; --clean) args+=(-Clean) ;;
        *) args+=("$a") ;;
      esac
    done
    exec powershell -ExecutionPolicy Bypass -File scripts/build-windows.ps1 ${args[@]+"${args[@]}"} ;;
  *) echo "Unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac
