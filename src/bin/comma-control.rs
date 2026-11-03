use comma_sound::{data_dir, State};
use std::{
    env,
    process::{Command, Stdio},
};

#[cfg(target_os = "linux")]
fn comma_for_key(code: u32, level: u32) -> Result<bool, String> {
    let output = Command::new("hyprctl")
        .args(["devices", "-j"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("hyprctl failed".into());
    }
    let data: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    comma_from_devices(&data, code, level)
}

#[cfg(target_os = "linux")]
fn comma_from_devices(data: &serde_json::Value, code: u32, level: u32) -> Result<bool, String> {
    use xkbcommon::xkb;
    let keyboard = data["keyboards"]
        .as_array()
        .ok_or("missing keyboards")?
        .iter()
        .find(|k| k["main"].as_bool() == Some(true))
        .ok_or("missing main keyboard")?;
    let get = |key: &str| keyboard[key].as_str().unwrap_or("");
    let group = keyboard["active_layout_index"]
        .as_u64()
        .ok_or("missing layout index")? as u32;
    let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
    let keymap = xkb::Keymap::new_from_names(
        &context,
        get("rules"),
        get("model"),
        get("layout"),
        get("variant"),
        Some(get("options").into()),
        xkb::KEYMAP_COMPILE_NO_FLAGS,
    )
    .ok_or("invalid keymap")?;
    Ok(keymap
        .key_get_syms_by_level(code.into(), group, level)
        .first()
        .copied()
        == Some(xkb::keysyms::KEY_comma.into()))
}
#[cfg(not(target_os = "linux"))]
fn comma_for_key(_: u32, _: u32) -> Result<bool, String> {
    Err("Hyprland integration requires Linux".into())
}

fn run() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();
    let command = args
        .get(1)
        .ok_or("usage: comma-control toggle|up|down|play KEYCODE LEVEL")?;
    let dir = data_dir();
    let path = dir.join("state");
    let mut state = State::read(&path);
    if state.change(command) {
        return state.write(&path).map_err(|e| e.to_string());
    }
    if command != "play" {
        return Err("unknown command".into());
    }
    if !state.enabled || state.volume == 0 {
        return Ok(());
    }
    let code = args
        .get(2)
        .ok_or("missing keycode")?
        .parse()
        .map_err(|_| "invalid keycode")?;
    let level = args
        .get(3)
        .ok_or("missing level")?
        .parse()
        .map_err(|_| "invalid level")?;
    if comma_for_key(code, level)? {
        Command::new("paplay")
            .arg(format!(
                "--volume={}",
                (65536_u32 * state.volume as u32 + 50) / 100
            ))
            .arg(dir.join("sound.wav"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("comma-control: {e}");
        std::process::exit(1);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn resolves_three_layouts_using_correct_level() {
        for (group, code, level, expected) in [
            (0, 59, 0, true),
            (0, 61, 1, false),
            (1, 61, 1, true),
            (1, 59, 0, false),
            (2, 61, 1, true),
            (2, 59, 0, false),
        ] {
            let devices = serde_json::json!({"keyboards": [{"main": true, "active_layout_index": group, "layout": "us,ru,ua"}]});
            assert_eq!(comma_from_devices(&devices, code, level).unwrap(), expected);
        }
    }
    #[test]
    fn no_keyboard_returns_error() {
        assert!(comma_from_devices(&serde_json::json!({"keyboards": []}), 59, 0).is_err());
    }
}
