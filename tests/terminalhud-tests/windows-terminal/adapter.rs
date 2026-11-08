use terminalhud_windows_terminal::*;
#[path = "cases/correlation.rs"]
mod correlation;
#[path = "cases/foreground.rs"]
mod foreground;
#[path = "cases/identity.rs"]
mod identity;
#[path = "cases/launch.rs"]
mod launch;
#[path = "cases/profiles.rs"]
mod profiles;
#[path = "cases/startup_control.rs"]
mod startup_control;

fn guid(suffix: u8) -> Guid {
    Guid::parse(&format!(
        "{{01234567-89ab-4cde-8fab-0123456789{suffix:02x}}}"
    ))
    .unwrap()
}
