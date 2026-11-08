//! Native menu/icon resources; actions only enqueue host commands.
use super::host::{UiMessage, UiSender};
use std::io;
use terminalhud_core::Command;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
pub fn tray(ui: UiSender) -> io::Result<TrayIcon> {
    let menu = Menu::new();
    for (id, label) in [
        ("interaction", "Toggle interaction (prototype)"),
        ("visibility", "Toggle visibility (prototype)"),
        ("passive", "Make Passive (prototype)"),
    ] {
        menu.append(&MenuItem::with_id(id, label, true, None))
            .map_err(io::Error::other)?;
    }
    menu.append(&PredefinedMenuItem::separator())
        .map_err(io::Error::other)?;
    menu.append(&MenuItem::with_id("quit", "Quit prototype", true, None))
        .map_err(io::Error::other)?;
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let command = match event.id().as_ref() {
            "interaction" => Command::InteractionToggle,
            "visibility" => Command::VisibilityToggle,
            "passive" => Command::Passive,
            "quit" => Command::Quit,
            _ => return,
        };
        let _ = ui.send(UiMessage::Request {
            command,
            reply: None,
        });
    }));
    let mut rgba = vec![0; 16 * 16 * 4];
    for y in 0..16 {
        for x in 0..16 {
            let p = (y * 16 + x) * 4;
            rgba[p..p + 4].copy_from_slice(if (3..13).contains(&x) && (3..13).contains(&y) {
                &[50, 180, 210, 255]
            } else {
                &[0, 0, 0, 0]
            });
        }
    }
    let icon = Icon::from_rgba(rgba, 16, 16).map_err(io::Error::other)?;
    TrayIconBuilder::new()
        .with_id("terminalhud-prototype")
        .with_menu(Box::new(menu))
        .with_tooltip("TerminalHUD prototype — no terminal attached")
        .with_icon(icon)
        .build()
        .map_err(io::Error::other)
}
