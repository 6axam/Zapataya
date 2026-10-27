#!/bin/sh
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
mkdir -p "$HOME/.local/share/comma-sound" "$HOME/.config/omarchy/plugins/maks.comma-sound"
cp "$HERE/control.py" "$HOME/.local/share/comma-sound/control.py"
cp "$HERE/../assets/sound.wav" "$HOME/.local/share/comma-sound/sound.wav"
chmod +x "$HOME/.local/share/comma-sound/control.py"
[ -e "$HOME/.local/share/comma-sound/state" ] || echo '1 50' > "$HOME/.local/share/comma-sound/state"
cp "$HERE/manifest.json" "$HERE/BarWidget.qml" "$HOME/.config/omarchy/plugins/maks.comma-sound/"
echo 'Установлены файлы. Добавьте привязки и виджет вручную по README.'
