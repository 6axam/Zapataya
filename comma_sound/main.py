"""Comma Sound: a small tray application that reacts only to typed commas."""
import os
import sys
from pathlib import Path

from PySide6.QtCore import QObject, QSettings, Signal, QUrl, Qt
from PySide6.QtGui import QAction, QColor, QIcon, QPainter, QPixmap
from PySide6.QtMultimedia import QSoundEffect
from PySide6.QtWidgets import QApplication, QLabel, QMenu, QMessageBox, QSlider, QSystemTrayIcon, QVBoxLayout, QWidget

BASE = Path(getattr(sys, '_MEIPASS', Path(__file__).resolve().parent.parent))

class Events(QObject):
    comma = Signal()

class Window(QWidget):
    def __init__(self):
        super().__init__()
        self.setWindowTitle('Comma Sound')
        self.settings = QSettings('CommaSound', 'CommaSound')
        self.enabled = self.settings.value('enabled', True, type=bool)
        self.volume = self.settings.value('volume', 50, type=int)
        self.sound = QSoundEffect(self)
        self.sound.setSource(QUrl.fromLocalFile(str(BASE / 'assets' / 'sound.wav')))
        self.sound.setVolume(self.volume / 100)
        self.events = Events()
        self.events.comma.connect(self.play)
        self.listener = None
        self.status = QLabel('Запустите мониторинг клавиатуры')
        self.volume_label = QLabel()
        self.slider = QSlider(Qt.Orientation.Horizontal)
        self.slider.setRange(0, 100)
        self.slider.setValue(self.volume)
        self.slider.valueChanged.connect(self.set_volume)
        layout = QVBoxLayout(self)
        layout.addWidget(self.status)
        layout.addWidget(self.volume_label)
        layout.addWidget(self.slider)
        self.resize(340, 130)
        self.tray = QSystemTrayIcon(self.make_icon(), self)
        self.menu = QMenu()
        self.toggle_action = QAction('Включено', self, checkable=True, checked=self.enabled)
        self.toggle_action.toggled.connect(self.set_enabled)
        self.menu.addAction(self.toggle_action)
        self.menu.addAction('Настройки', self.show_settings)
        self.menu.addAction('Проверить звук', self.preview)
        self.menu.addSeparator()
        self.menu.addAction('Выход', QApplication.quit)
        self.tray.setContextMenu(self.menu)
        self.tray.activated.connect(self.tray_activated)
        self.tray.show()
        self.update_status()
        self.start_listener()
        if not QSystemTrayIcon.isSystemTrayAvailable():
            self.show()

    @staticmethod
    def make_icon():
        pix = QPixmap(64, 64)
        pix.fill(Qt.GlobalColor.transparent)
        painter = QPainter(pix)
        painter.setRenderHint(QPainter.RenderHint.Antialiasing)
        painter.setBrush(QColor('#7564df'))
        painter.setPen(Qt.PenStyle.NoPen)
        painter.drawRoundedRect(0, 0, 64, 64, 15, 15)
        painter.setPen(QColor('white'))
        font = painter.font(); font.setBold(True); font.setPixelSize(50); painter.setFont(font)
        painter.drawText(pix.rect(), Qt.AlignmentFlag.AlignCenter, ',')
        painter.end()
        return QIcon(pix)

    def start_listener(self):
        if sys.platform.startswith('linux') and os.environ.get('XDG_SESSION_TYPE', '').lower() == 'wayland':
            self.status.setText('Wayland: используйте интеграцию Hyprland из README')
            self.show()
            return
        try:
            from pynput import keyboard
            mods = set()
            controls = {keyboard.Key.ctrl, keyboard.Key.ctrl_l, keyboard.Key.ctrl_r,
                        keyboard.Key.alt, keyboard.Key.alt_l, keyboard.Key.alt_r,
                        keyboard.Key.cmd, keyboard.Key.cmd_l, keyboard.Key.cmd_r}
            def press(key):
                if key in controls:
                    mods.add(key)
                elif not mods and getattr(key, 'char', None) == ',':
                    self.events.comma.emit()
            def release(key):
                mods.discard(key)
            self.listener = keyboard.Listener(on_press=press, on_release=release)
            self.listener.start()
            if sys.platform == 'darwin' and not self.listener.IS_TRUSTED:
                self.status.setText('macOS: разрешите мониторинг ввода и Универсальный доступ')
                self.show()
            else:
                self.status.setText('Мониторинг клавиатуры включён')
        except Exception as error:
            self.status.setText('Не удалось запустить мониторинг: ' + str(error))
            self.show()

    def play(self):
        if self.enabled:
            self.sound.stop()
            self.sound.play()

    def preview(self):
        self.sound.stop()
        self.sound.play()

    def set_enabled(self, value):
        self.enabled = value
        self.settings.setValue('enabled', value)
        self.update_status()

    def set_volume(self, value):
        self.volume = value
        self.sound.setVolume(value / 100)
        self.settings.setValue('volume', value)
        self.update_status()

    def update_status(self):
        self.volume_label.setText(f'Громкость: {self.volume}%')
        self.tray.setToolTip(f'Comma Sound — {self.volume}%' if self.enabled else 'Comma Sound — выключено')
        self.toggle_action.setText('Включено' if self.enabled else 'Выключено')

    def show_settings(self):
        self.show(); self.raise_(); self.activateWindow()

    def tray_activated(self, reason):
        if reason == QSystemTrayIcon.ActivationReason.Trigger:
            self.show_settings()

    def closeEvent(self, event):
        if QSystemTrayIcon.isSystemTrayAvailable():
            event.ignore(); self.hide()
        else:
            event.accept(); QApplication.quit()


def main():
    if '--self-test' in sys.argv:
        os.environ.setdefault('QT_QPA_PLATFORM', 'offscreen')
        app = QApplication([sys.argv[0]])
        effect = QSoundEffect()
        path = BASE / 'assets' / 'sound.wav'
        if not path.is_file():
            return 1
        effect.setSource(QUrl.fromLocalFile(str(path)))
        print('Self-test OK: Qt and bundled sound loaded')
        return 0
    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)
    window = Window()
    app.setWindowIcon(window.make_icon())
    window.show()
    result = app.exec()
    if window.listener:
        window.listener.stop()
    return result

if __name__ == '__main__':
    sys.exit(main())
