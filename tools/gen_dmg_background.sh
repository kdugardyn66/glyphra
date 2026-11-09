#!/usr/bin/env bash
# Render packaging/macos/dmg-background.svg to the 1x and 2x PNGs the macOS build uses.
# The PNGs carry a DPI (72 and 144): `tiffutil -cathidpicheck` refuses the pair without it.
# Needs rsvg-convert and Python with Pillow.   Usage: tools/gen_dmg_background.sh
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../packaging/macos"
rsvg-convert -w 640 -h 400 dmg-background.svg -o /tmp/dmg-1x.png
rsvg-convert -w 1280 -h 800 dmg-background.svg -o /tmp/dmg-2x.png
python3 - <<'PY'
from PIL import Image
for src, dst, dpi in (("/tmp/dmg-1x.png", "dmg-background.png", 72), ("/tmp/dmg-2x.png", "dmg-background@2x.png", 144)):
    Image.open(src).convert("RGB").save(dst, dpi=(dpi, dpi), optimize=True)
PY
echo "wrote packaging/macos/dmg-background.png and dmg-background@2x.png"
