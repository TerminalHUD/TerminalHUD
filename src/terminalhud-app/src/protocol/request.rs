use super::REQUEST_LEN;
use terminalhud_core::Command;

/// Application requests are routed before the portable window controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Command(Command),
    ReloadConfiguration,
}
impl From<Command> for Request {
    fn from(command: Command) -> Self {
        Self::Command(command)
    }
}
pub fn encode_request(request: impl Into<Request>) -> [u8; REQUEST_LEN] {
    let mut frame = [0; REQUEST_LEN];
    frame[..4].copy_from_slice(b"THP1");
    frame[4] = match request.into() {
        Request::Command(Command::InteractionToggle) => 1,
        Request::Command(Command::VisibilityToggle) => 2,
        Request::Command(Command::Passive) => 3,
        Request::Command(Command::Status) => 4,
        Request::Command(Command::Quit) => 5,
        Request::ReloadConfiguration => 6,
    };
    frame
}

pub fn decode_request(frame: &[u8]) -> Result<Request, &'static str> {
    if frame.len() != REQUEST_LEN || &frame[..4] != b"THP1" || frame[5..].iter().any(|v| *v != 0) {
        return Err("invalid prototype request length, version, or reserved fields");
    }
    match frame[4] {
        1 => Ok(Command::InteractionToggle.into()),
        2 => Ok(Command::VisibilityToggle.into()),
        3 => Ok(Command::Passive.into()),
        4 => Ok(Command::Status.into()),
        5 => Ok(Command::Quit.into()),
        6 => Ok(Request::ReloadConfiguration),
        _ => Err("unknown prototype command"),
    }
}
