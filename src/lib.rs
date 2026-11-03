use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub enabled: bool,
    pub volume: u8,
}
impl Default for State {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 50,
        }
    }
}
impl State {
    pub fn parse(s: &str) -> Option<Self> {
        let mut v = s.split_whitespace();
        let enabled = match v.next()? {
            "1" => true,
            "0" => false,
            _ => return None,
        };
        let volume: u8 = v.next()?.parse().ok()?;
        if volume > 100 || v.next().is_some() {
            return None;
        }
        Some(Self { enabled, volume })
    }
    pub fn change(&mut self, cmd: &str) -> bool {
        match cmd {
            "toggle" => self.enabled = !self.enabled,
            "up" => self.volume = self.volume.saturating_add(10).min(100),
            "down" => self.volume = self.volume.saturating_sub(10),
            _ => return false,
        }
        true
    }
    pub fn read(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| Self::parse(&s))
            .unwrap_or_default()
    }
    pub fn write(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(
            path,
            format!("{} {}\n", u8::from(self.enabled), self.volume),
        )
    }
}
pub fn data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("COMMA_SOUND_HOME") {
        return PathBuf::from(path);
    }
    #[cfg(target_os = "linux")]
    {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".local/share/comma-sound")
    }
    #[cfg(not(target_os = "linux"))]
    {
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("CommaSound")
    }
}
pub fn is_comma(name: Option<&str>, blocked: bool) -> bool {
    !blocked && name == Some(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_legacy_and_validation() {
        assert_eq!(
            State::parse("1 90\n"),
            Some(State {
                enabled: true,
                volume: 90
            })
        );
        assert!(State::parse("2 50").is_none());
        assert!(State::parse("0 101").is_none());
        assert!(State::parse("1 50 0").is_none());
    }
    #[test]
    fn volume_and_toggle() {
        let mut s = State {
            enabled: true,
            volume: 95,
        };
        s.change("up");
        assert_eq!(s.volume, 100);
        for _ in 0..12 {
            s.change("down");
        }
        assert_eq!(s.volume, 0);
        s.change("toggle");
        assert!(!s.enabled);
        assert!(!s.change("bad"));
    }
    #[test]
    fn comma_only_without_control_modifiers() {
        assert!(is_comma(Some(","), false));
        assert!(!is_comma(Some(","), true));
        assert!(!is_comma(Some("."), false));
        assert!(!is_comma(None, false));
    }
}
