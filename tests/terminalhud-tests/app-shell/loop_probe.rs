use std::{sync::mpsc, time::Duration};
use terminalhud_app::windows::{self, UiSender};
use windows_sys::Win32::{System::Threading::GetCurrentThreadId, UI::WindowsAndMessaging::*};

pub(super) fn native() -> (u32, bool) {
    let (sender, _queue) = mpsc::sync_channel(8);
    let thread_id = unsafe { GetCurrentThreadId() };
    let mut message: MSG = unsafe { std::mem::zeroed() };
    unsafe {
        PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
    }
    let icon = windows::tray(UiSender::new(sender, 0)).unwrap();
    let rect = icon.rect().is_some();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        unsafe {
            PostThreadMessageW(thread_id, WM_APP + 9, 0, 0);
        }
        std::thread::sleep(Duration::from_millis(650));
        unsafe {
            PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
        }
    });
    let mut events = 0;
    while unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) } > 0 {
        if message.message == WM_APP + 9 {
            events += 1;
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    drop(icon);
    (events, rect)
}

pub(super) fn tao() -> (u32, bool) {
    use tao::{
        event::Event,
        event_loop::{ControlFlow, EventLoopBuilder},
        platform::run_return::EventLoopExtRunReturn,
    };
    let (sender, _queue) = mpsc::sync_channel(8);
    let mut event_loop = EventLoopBuilder::<u8>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let icon = windows::tray(UiSender::new(sender, 0)).unwrap();
    let rect = icon.rect().is_some();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        proxy.send_event(1).unwrap();
        std::thread::sleep(Duration::from_millis(650));
        proxy.send_event(2).unwrap();
    });
    let mut events = 0;
    event_loop.run_return(|event, _, flow| {
        *flow = ControlFlow::Wait;
        if let Event::UserEvent(value) = event {
            if value == 1 {
                events += 1;
            } else {
                *flow = ControlFlow::Exit;
            }
        }
    });
    drop(icon);
    (events, rect)
}
