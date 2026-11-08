use super::REPLY_LEN;
use terminalhud_core::integration::{Availability, State, Status, VisibleState};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyCode {
    Ok,
    Malformed,
    TransitionFailed,
    ShuttingDown,
}

#[derive(Clone, Copy, Debug)]
pub struct Reply {
    pub code: ReplyCode,
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
        frame[..4].copy_from_slice(b"THR1");
        frame[4] = match self.code {
            ReplyCode::Ok => 0,
            ReplyCode::Malformed => 1,
            ReplyCode::TransitionFailed => 2,
            ReplyCode::ShuttingDown => 3,
        };
        frame[5] = 1; // Explicit prototype mode, never a real Terminal HUD claim.
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
            || &frame[..4] != b"THR1"
            || frame[5] != 1
            || frame[8] > 1
            || frame[9] > 1
            || frame[10..12]
                .iter()
                .chain(frame[28..32].iter())
                .any(|v| *v != 0)
        {
            return Err("invalid prototype reply");
        }
        let code = match frame[4] {
            0 => ReplyCode::Ok,
            1 => ReplyCode::Malformed,
            2 => ReplyCode::TransitionFailed,
            3 => ReplyCode::ShuttingDown,
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
            "mode=prototype pid={} availability={:?} state={:?} result={:?} focus_warning={} host_console={} sequence={} menu_commands={}",
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
