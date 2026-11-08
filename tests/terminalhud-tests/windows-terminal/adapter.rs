use terminalhud_windows_terminal::*;
#[path = "cases/active_render.rs"]
mod active_render;
#[path = "cases/correlation.rs"]
mod correlation;
#[path = "cases/identity.rs"]
mod identity;
#[path = "cases/input_recovery.rs"]
mod input_recovery;
#[path = "cases/launch.rs"]
mod launch;
#[path = "cases/output_marker.rs"]
mod output_marker;
#[path = "cases/profiles.rs"]
mod profiles;
#[path = "cases/underlay.rs"]
mod underlay;

fn guid(suffix: u8) -> Guid {
    Guid::parse(&format!(
        "{{01234567-89ab-4cde-8fab-0123456789{suffix:02x}}}"
    ))
    .unwrap()
}
