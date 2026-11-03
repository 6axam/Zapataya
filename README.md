# Comma Sound

Приложение на Rust воспроизводит звук при вводе запятой. Через значок в трее можно включить и выключить звук, менять громкость и проверить её. Функция не записывает и не передаёт набираемый текст.

## Скачать

На странице Releases выберите Windows `.exe`, архив `.app` для macOS (Apple Silicon или Intel) либо файл или пакет для Linux x86-64. На Linux доступны пакеты DEB (Debian/Ubuntu), RPM (Fedora/openSUSE), pacman (Arch/Omarchy) и архив. Для отдельного файла Linux потребуется `chmod +x CommaSound-linux-x64`; ему нужны системные библиотеки GTK3, ALSA, X11, XTest и AppIndicator.

## Возможности и ограничения

- Windows: программа работает из трея. Файл не подписан, пока автор не настроит доверенную подпись, и SmartScreen может предупредить о неизвестном издателе. Окна, запущенные с повышенными правами, могут быть недоступны.
- macOS: разрешите программе «Мониторинг ввода» и «Универсальный доступ». Сборка не подписана и не нотариализована.
- Linux X11: нужны X-сервер и системный трей. В некоторых окружениях требуется поддержка AppIndicator.
- Linux Wayland не предоставляет универсальный мониторинг клавиатуры приложениям. Для Omarchy/Hyprland есть интеграция ниже; другие композиторы Wayland не поддерживаются.
- Ввод через экранную клавиатуру, IME, вставку, удалённый рабочий стол и специализированные приложения зависит от системы и может не распознаваться. Комбинации с Ctrl, Alt или Super не воспроизводят звук.

## Omarchy + Hyprland

Интеграция рассчитана на раскладки US, RU и UA с US первой в списке. Она использует виджет Omarchy, а не приложение из трея. Распакуйте Linux-архив или соберите проект из исходников, затем выполните:

```sh
./hyprland/install.sh
```

Добавьте в `~/.config/hypr/bindings.lua` (замените `/home/USER` на свой домашний каталог):

```lua
hl.bind("comma", hl.dsp.exec_cmd("/home/USER/.local/share/comma-sound/comma-control play 59 0"), { non_consuming = true, repeating = true })
hl.bind("SHIFT + slash", hl.dsp.exec_cmd("/home/USER/.local/share/comma-sound/comma-control play 61 1"), { non_consuming = true, repeating = true })
```

Если ранее установлена Python-версия, **замените** старые привязки, не добавляйте рядом дубликаты. Добавьте виджет `omarchy bar put maks.comma-sound --section right`, если его ещё нет; затем запустите `hyprctl reload` и `hyprctl configerrors`. Клик по виджету переключает звук, колесо регулирует громкость. Файл состояния совместим с предыдущей версией. Нужны `hyprctl`, `paplay` и системная библиотека `libxkbcommon`.

## Сборка и тестирование

Установите Rust и системные библиотеки GTK3, ALSA, X11, XTest, xkbcommon и AppIndicator для Linux, затем:

```sh
cargo fmt --all --check
cargo test --locked --all-targets
cargo build --locked --release --bins
```

Программа: `target/release/comma-sound`; обработчик Hyprland: `target/release/comma-control`. Windows и macOS собираются отдельно на соответствующей платформе.

## Подпись Windows

Текущий релиз не подписан. SmartScreen может показать «Windows защитила ваш компьютер» и неизвестного издателя. Для подписи будущего релиза необходим доверенный сертификат Code Signing; самоподписанный сертификат не устранит предупреждение. Сертификаты с неэкспортируемым ключом или облачной подписью требуют отдельной интеграции поставщика. Для экспортируемого PFX настройте GitHub Actions secrets `WINDOWS_CERTIFICATE_PFX_BASE64` и `WINDOWS_CERTIFICATE_PASSWORD`, а переменную `WINDOWS_SIGNING_ENABLED` установите в `true`. Если настройки неверны, Windows-сборка завершится с ошибкой. Не размещайте ключи и пароль в репозитории. Подпись не гарантирует немедленного исчезновения предупреждения: [Microsoft о репутации SmartScreen](https://learn.microsoft.com/windows/apps/package-and-deploy/smartscreen-reputation).

Звук предоставлен владельцем репозитория; права на его распространение определяются правообладателем.
