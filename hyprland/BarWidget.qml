import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui

BarWidget {
  id: root
  moduleName: "maks.comma-sound"
  property bool enabledSound: true
  property int volume: 50
  readonly property string control: Quickshell.env("HOME") + "/.local/share/comma-sound/control.py"
  implicitWidth: button.implicitWidth
  implicitHeight: button.implicitHeight
  function refresh() { if (!readState.running) readState.running = true }
  function change(action) {
    if (!changeState.running) { changeState.command = [control, action]; changeState.running = true }
  }
  Component.onCompleted: refresh()
  Timer { interval: 2000; running: true; repeat: true; onTriggered: root.refresh() }
  Process {
    id: readState
    command: ["cat", Quickshell.env("HOME") + "/.local/share/comma-sound/state"]
    stdout: StdioCollector { onStreamFinished: { var s = text.trim().split(/\s+/); if (s.length === 2) { root.enabledSound = s[0] === "1"; root.volume = Number(s[1]) } } }
  }
  Process { id: changeState; onExited: root.refresh() }
  BarIconButton {
    id: button
    anchors.fill: parent
    bar: root.bar
    text: root.enabledSound ? "󰝚" : "󰖁"
    tooltipText: "Звук запятой: " + (root.enabledSound ? root.volume + "%" : "выключен") + " · клик: вкл/выкл · колесо: громкость"
    onPressed: function(b) { root.change("toggle") }
    onWheelMoved: function(delta) { root.change(delta > 0 ? "up" : "down") }
  }
}
