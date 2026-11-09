use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Modifier {
    Win,
    Ctrl,
    Alt,
    Shift,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Backtick,
    Letter(char),
    Digit(u8),
    Function(u8),
}

/// A supported symbolic chord, not a native keycode or registration guarantee.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shortcut {
    modifiers: Vec<Modifier>,
    key: Key,
}

impl Shortcut {
    pub fn parse(value: &str) -> Result<Self, String> {
        let mut parts: Vec<_> = value.split('+').collect();
        let key = match parts.pop().unwrap_or("") {
            "Backtick" => Key::Backtick,
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_uppercase() => {
                Key::Letter(s.chars().next().unwrap())
            }
            s if s.len() == 1 && s.as_bytes()[0].is_ascii_digit() => {
                Key::Digit(s.as_bytes()[0] - b'0')
            }
            s => {
                let number = s.strip_prefix('F').and_then(|n| n.parse::<u8>().ok());
                match number {
                    Some(n @ 1..=24) if s == format!("F{n}") => Key::Function(n),
                    _ => {
                        return Err(
                            "expected Backtick, A..Z, 0..9 or F1..F24 after modifiers".into()
                        );
                    }
                }
            }
        };
        let mut modifiers = Vec::new();
        for part in parts {
            let modifier = match part {
                "Win" => Modifier::Win,
                "Ctrl" => Modifier::Ctrl,
                "Alt" => Modifier::Alt,
                "Shift" => Modifier::Shift,
                _ => return Err(format!("unsupported modifier {part:?}")),
            };
            if modifiers.contains(&modifier) {
                return Err("duplicate modifier".into());
            }
            modifiers.push(modifier);
        }
        if modifiers.is_empty() {
            return Err("at least one modifier is required".into());
        }
        modifiers.sort();
        Ok(Self { modifiers, key })
    }

    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }
    pub fn key(&self) -> Key {
        self.key
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for modifier in &self.modifiers {
            write!(f, "{modifier:?}+")?;
        }
        match self.key {
            Key::Backtick => write!(f, "Backtick"),
            Key::Letter(c) => write!(f, "{c}"),
            Key::Digit(n) => write!(f, "{n}"),
            Key::Function(n) => write!(f, "F{n}"),
        }
    }
}
