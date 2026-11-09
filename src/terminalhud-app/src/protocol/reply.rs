use super::REPLY_LEN;
use crate::mode::Mode;
use terminalhud_core::integration::{Availability, State, Status, VisibleState};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyCode {
    Ok,
    Malformed,
    TransitionFailed,
    ShuttingDown,
    RequiresRelaunch,
}

#[derive(Clone, Copy, Debug)]
pub struct Reply {
    pub code: ReplyCode,
    pub mode: Mode,
    /// Zero means both intended bindings registered; otherwise native failure code.
    pub hotkey_error: u32,
    pub status: Status,
    pub focus_warning: bool,
    pub sequence: u64,
    pub menu_commands: u32,
    pub process_id: u32,
    pub host_console: bool,
}

impl Reply {
    pub fn encode(self) -> [u8; REPLY_LEN] {
        let mut frame = [0; REPLY_LEN];
        frame[..4].copy_from_slice(b"THR2");
        frame[4] = match self.code {
            ReplyCode::Ok => 0,
            ReplyCode::Malformed => 1,
            ReplyCode::TransitionFailed => 2,
            ReplyCode::ShuttingDown => 3,
            ReplyCode::RequiresRelaunch => 4,
        };
        frame[5] = match self.mode {
            Mode::Prototype => 1,
            Mode::Real => 2,
        };
        frame[28..32].copy_from_slice(&self.hotkey_error.to_le_bytes());
        frame[6] = match self.status.availability {
            Availability::Absent => 0,
            Availability::Managed => 1,
            Availability::Unmanaged => 2,
        };
        frame[7] = match self.status.state {
            None => 0,
            Some(State::Active) => 1,
            Some(State::Passive) => 2,
            Some(State::Hidden {
                return_state: VisibleState::Active,
            }) => 3,
            Some(State::Hidden {
                return_state: VisibleState::Passive,
            }) => 4,
        };
        frame[8] = u8::from(self.focus_warning);
        frame[9] = u8::from(self.host_console);
        frame[12..20].copy_from_slice(&self.sequence.to_le_bytes());
        frame[20..24].copy_from_slice(&self.menu_commands.to_le_bytes());
        frame[24..28].copy_from_slice(&self.process_id.to_le_bytes());
        frame
    }

    pub fn decode(frame: &[u8]) -> Result<Self, &'static str> {
        if frame.len() != REPLY_LEN
            || &frame[..4] != b"THR2"
            || !(1..=2).contains(&frame[5])
            || frame[8] > 1
            || frame[9] > 1
            || frame[10..12].iter().any(|v| *v != 0)
        {
            return Err("invalid runtime reply");
        }
        let code = match frame[4] {
            0 => ReplyCode::Ok,
            1 => ReplyCode::Malformed,
            2 => ReplyCode::TransitionFailed,
            3 => ReplyCode::ShuttingDown,
            4 => ReplyCode::RequiresRelaunch,
            _ => return Err("unknown reply code"),
        };
        let availability = match frame[6] {
            0 => Availability::Absent,
            1 => Availability::Managed,
            2 => Availability::Unmanaged,
            _ => return Err("unknown availability"),
        };
        let state = match frame[7] {
            0 => None,
            1 => Some(State::Active),
            2 => Some(State::Passive),
            3 => Some(State::Hidden {
                return_state: VisibleState::Active,
            }),
            4 => Some(State::Hidden {
                return_state: VisibleState::Passive,
            }),
            _ => return Err("unknown state"),
        };
        if availability != Availability::Managed && state.is_some() {
            return Err("state on an absent/unmanaged reply");
        }
        Ok(Self {
            code,
            mode: if frame[5] == 1 {
                Mode::Prototype
            } else {
                Mode::Real
            },
            hotkey_error: u32::from_le_bytes(frame[28..32].try_into().unwrap()),
            status: Status {
                availability,
                state,
                target_id: None,
            },
            focus_warning: frame[8] == 1,
            host_console: frame[9] == 1,
            sequence: u64::from_le_bytes(frame[12..20].try_into().unwrap()),
            menu_commands: u32::from_le_bytes(frame[20..24].try_into().unwrap()),
            process_id: u32::from_le_bytes(frame[24..28].try_into().unwrap()),
        })
    }

    pub fn display(self) -> String {
        format!(
            "mode={} hotkey_error={} pid={} availability={:?} state={:?} result={:?} focus_warning={} host_console={} sequence={} menu_commands={}",
            self.mode.name(),
            self.hotkey_error,
            self.process_id,
            self.status.availability,
            self.status.state,
            self.code,
            self.focus_warning,
            self.host_console,
            self.sequence,
            self.menu_commands
        )
    }
}
