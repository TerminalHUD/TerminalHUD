use super::REQUEST_LEN;
use terminalhud_core::Command;
pub fn encode_request(command: Command) -> [u8; REQUEST_LEN] {
    let mut frame = [0; REQUEST_LEN];
    frame[..4].copy_from_slice(b"THP1");
    frame[4] = match command {
        Command::InteractionToggle => 1,
        Command::VisibilityToggle => 2,
        Command::Passive => 3,
        Command::Status => 4,
        Command::Quit => 5,
    };
    frame
}

pub fn decode_request(frame: &[u8]) -> Result<Command, &'static str> {
    if frame.len() != REQUEST_LEN || &frame[..4] != b"THP1" || frame[5..].iter().any(|v| *v != 0) {
        return Err("invalid prototype request length, version, or reserved fields");
    }
    match frame[4] {
        1 => Ok(Command::InteractionToggle),
        2 => Ok(Command::VisibilityToggle),
        3 => Ok(Command::Passive),
        4 => Ok(Command::Status),
        5 => Ok(Command::Quit),
        _ => Err("unknown prototype command"),
    }
}
