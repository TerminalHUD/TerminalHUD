use super::{fixture::Primary, invoke_menu, owned_windows};
use std::{
    thread,
    time::{Duration, Instant},
};
use terminalhud_core::{
    Command,
    integration::{Availability, State},
};

#[test]
#[ignore = "visible owned tray/native menu proof; serialize with other desktop work"]
fn tray_menu_and_ipc_remain_responsive_in_modal_loop() {
    let mut primary = Primary::start_visible();
    thread::sleep(Duration::from_millis(250));
    let initial = primary.call(Command::Status);
    assert!(
        !initial.host_console,
        "GUI host attached/allocated a console"
    );
    assert!(
        owned_windows::visible(primary.pid()).is_empty(),
        "unexpected visible app window"
    );
    println!("opening owned native popup");
    owned_windows::open_menu(primary.pid());
    let started = Instant::now();
    let reply = primary.call(Command::VisibilityToggle);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "IPC stalled in menu loop"
    );
    assert_eq!(reply.status.state, Some(State::Passive));
    println!("IPC while modal confirmed");
    let menu = owned_windows::wait_menu(primary.pid());
    invoke_menu::interaction(primary.pid(), menu as usize);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let status = primary.call(Command::Status);
        if status.menu_commands == 1 {
            assert_eq!(status.status.state, Some(State::Active));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "native menu action not dispatched"
        );
        thread::sleep(Duration::from_millis(10));
    }
    println!("native menu selection confirmed exactly once");
    // A quit reply must be consumed before closing the active native menu/host.
    owned_windows::open_menu(primary.pid());
    let reply = primary.call(Command::Quit);
    assert_eq!(reply.status.availability, Availability::Unmanaged);
    primary.wait();
    assert!(owned_windows::visible(primary.pid()).is_empty());
    println!(
        "native tray: no console/main window; IPC while modal; owned menu dispatched; quit reply consumed; host cleaned"
    );
}
