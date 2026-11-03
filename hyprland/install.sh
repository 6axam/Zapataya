#!/bin/sh
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(CDPATH= cd -- "$HERE/.." && pwd)
if [ -x "$ROOT/comma-control" ]; then CONTROL="$ROOT/comma-control"; elif [ -x "$ROOT/target/release/comma-control" ]; then CONTROL="$ROOT/target/release/comma-control"; elif command -v comma-control >/dev/null 2>&1; then CONTROL=$(command -v comma-control); else echo 'Сначала соберите проект: cargo build --release --bins' >&2; exit 1; fi
mkdir -p "$HOME/.local/share/comma-sound" "$HOME/.config/omarchy/plugins/maks.comma-sound"
install -m755 "$CONTROL" "$HOME/.local/share/comma-sound/comma-control"
cp "$ROOT/assets/sound.wav" "$HOME/.local/share/comma-sound/sound.wav"
[ -e "$HOME/.local/share/comma-sound/state" ] || echo '1 50' > "$HOME/.local/share/comma-sound/state"
cp "$HERE/manifest.json" "$HERE/BarWidget.qml" "$HOME/.config/omarchy/plugins/maks.comma-sound/"
echo 'Добавьте привязки и виджет по README; старые привязки на control.py замените.'
