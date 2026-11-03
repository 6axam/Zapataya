#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use comma_sound::{data_dir, is_comma, State};
use std::{
    collections::HashSet,
    io::Cursor,
    sync::{mpsc, Arc, Mutex},
};
use tao::{
    event::{Event, StartCause},
    event_loop::{ControlFlow, EventLoopBuilder},
};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

const SOUND: &[u8] = include_bytes!("../assets/sound.wav");
enum UserEvent {
    Menu(tray_icon::menu::MenuEvent),
}

fn icon() -> tray_icon::Icon {
    let mut rgba = vec![0u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let i = (y * 32 + x) * 4;
            let white = (13..=18).contains(&x) && (15..=23).contains(&y)
                || (11..=15).contains(&x) && (22..=26).contains(&y);
            rgba[i..i + 4].copy_from_slice(if white {
                &[255, 255, 255, 255]
            } else {
                &[117, 100, 223, 255]
            });
        }
    }
    tray_icon::Icon::from_rgba(rgba, 32, 32).expect("valid icon")
}
fn audio_thread(rx: mpsc::Receiver<u8>) {
    let Ok((_stream, handle)) = rodio::OutputStream::try_default() else {
        eprintln!("No audio output available");
        return;
    };
    let mut previous: Option<rodio::Sink> = None;
    while let Ok(volume) = rx.recv() {
        if let Some(sink) = previous.take() {
            sink.stop();
        }
        if let (Ok(sink), Ok(source)) = (
            rodio::Sink::try_new(&handle),
            rodio::Decoder::new(Cursor::new(SOUND)),
        ) {
            sink.set_volume(volume as f32 / 100.0);
            sink.append(source);
            previous = Some(sink);
        }
    }
}
fn main() {
    if std::env::args().any(|a| a == "--self-test") {
        assert!(SOUND.starts_with(b"RIFF") && &SOUND[8..12] == b"WAVE");
        println!("Self-test OK: embedded WAV");
        return;
    }
    let path = data_dir().join("state");
    let state = Arc::new(Mutex::new(State::read(&path)));
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || audio_thread(rx));
    let wayland = cfg!(target_os = "linux")
        && std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_default()
            .eq_ignore_ascii_case("wayland");
    if !wayland {
        let state = state.clone();
        let tx = tx.clone();
        std::thread::spawn(move || {
            use rdev::{EventType, Key};
            let mut mods = HashSet::new();
            let result = rdev::listen(move |event| {
                let modifier = |k| {
                    matches!(
                        k,
                        Key::ControlLeft
                            | Key::ControlRight
                            | Key::Alt
                            | Key::AltGr
                            | Key::MetaLeft
                            | Key::MetaRight
                    )
                };
                match event.event_type {
                    EventType::KeyPress(k) if modifier(k) => {
                        mods.insert(k);
                    }
                    EventType::KeyRelease(k) if modifier(k) => {
                        mods.remove(&k);
                    }
                    EventType::KeyPress(_) if is_comma(event.name.as_deref(), !mods.is_empty()) => {
                        if let Ok(s) = state.lock() {
                            if s.enabled && s.volume > 0 {
                                let _ = tx.send(s.volume);
                            }
                        }
                    }
                    _ => {}
                }
            });
            if let Err(e) = result {
                eprintln!("Keyboard listener failed: {e:?}");
            }
        });
    } else {
        eprintln!("Wayland: use the Hyprland integration described in README");
    }
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));
    let menu = Menu::new();
    let status = MenuItem::new(
        if wayland {
            "Wayland: используйте интеграцию Hyprland"
        } else {
            "Comma Sound"
        },
        false,
        None,
    );
    let toggle = MenuItem::new("", true, None);
    let volume = MenuItem::new("", false, None);
    let up = MenuItem::new("Громче +10%", true, None);
    let down = MenuItem::new("Тише −10%", true, None);
    let preview = MenuItem::new("Проверить звук", true, None);
    let quit = MenuItem::new("Выход", true, None);
    menu.append_items(&[
        &status,
        &toggle,
        &volume,
        &up,
        &down,
        &preview,
        &PredefinedMenuItem::separator(),
        &quit,
    ])
    .expect("menu");
    let toggle_label = toggle.clone();
    let volume_label = volume.clone();
    let refresh = move |s: State| {
        toggle_label.set_text(if s.enabled {
            "Выключить звук"
        } else {
            "Включить звук"
        });
        volume_label.set_text(format!("Громкость: {}%", s.volume));
    };
    refresh(*state.lock().expect("state"));
    let mut tray = None;
    event_loop.run(move |event, _, control| {
        *control = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {
                match TrayIconBuilder::new()
                    .with_menu(Box::new(menu.clone()))
                    .with_tooltip("Comma Sound")
                    .with_icon(icon())
                    .build()
                {
                    Ok(t) => tray = Some(t),
                    Err(e) => {
                        eprintln!("Could not create tray icon: {e}");
                        *control = ControlFlow::Exit;
                    }
                }
            }
            Event::UserEvent(UserEvent::Menu(event)) => {
                if event.id == *quit.id() {
                    tray.take();
                    *control = ControlFlow::Exit;
                    return;
                }
                let mut s = state.lock().expect("state");
                if event.id == *preview.id() {
                    let _ = tx.send(s.volume);
                    return;
                }
                let cmd = if event.id == *toggle.id() {
                    "toggle"
                } else if event.id == *up.id() {
                    "up"
                } else if event.id == *down.id() {
                    "down"
                } else {
                    return;
                };
                if s.change(cmd) {
                    if let Err(e) = s.write(&path) {
                        eprintln!("Could not save settings: {e}");
                    }
                    refresh(*s);
                }
            }
            _ => {}
        }
    });
}
