use super::super::{
    ipc::{self, Server},
    native::last,
    security::Identity,
    tray::tray,
};
use super::{HostOptions, TIMER_DURATION, UiSender, context::HostContext, window::HostWindow};
use std::{
    io, mem, ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use terminalhud_core::Command;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
static HOST_STARTED: AtomicBool = AtomicBool::new(false);
pub fn run_host(options: HostOptions) -> io::Result<()> {
    if HOST_STARTED.swap(true, Ordering::Relaxed) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "one GUI host per process",
        ));
    }
    let identity = Identity::current()?;
    let mut server = match Server::bind(&identity, &options.instance) {
        Ok(server) => server,
        Err(bind_error) => {
            return ipc::call(&options.instance, Command::Status)
                .map(|_| ())
                .map_err(|_| bind_error);
        }
    };
    let (sender, queue) = mpsc::sync_channel(64);
    let context = Box::new(HostContext::new(queue));
    let window = HostWindow::create(&context)?;
    context.window.set(window.0);
    context.menu_probe.set(options.menu_probe);
    let ui = UiSender::new(sender, window.0 as usize);
    if !options.headless {
        *context.tray.borrow_mut() = Some(tray(ui.clone())?);
    }
    if let Some(duration) = options.exit_after
        && unsafe {
            SetTimer(
                window.0,
                TIMER_DURATION,
                duration.as_millis().clamp(100, 60_000) as u32,
                None,
            )
        } == 0
    {
        return Err(last("SetTimer duration"));
    }
    // Readiness is serviced only after the UI/tray/controller has been initialized.
    server.start(identity, ui.clone())?;
    let mut message: MSG = unsafe { mem::zeroed() };
    let run_result = loop {
        let result = unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) };
        if result == -1 {
            break Err(last("GetMessage"));
        }
        if result == 0 {
            break Ok(());
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    };
    // Workers are cancelled/joined before removing their wake target. The server
    // retains its first-instance pipe handle until after the icon/window disappear.
    ui.stop();
    server.stop();
    drop(context.tray.borrow_mut().take());
    drop(window);
    drop(server);
    let context_error = context.error.borrow_mut().take();
    run_result.and_then(|()| context_error.map_or(Ok(()), Err))
}
