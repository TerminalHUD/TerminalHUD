use super::super::{
    ipc::{self, Server},
    security::Identity,
    tray::tray_for_mode,
};
use super::{
    HostOptions, UiSender, context::HostContext, controller::RuntimeController, hotkeys,
    window::HostWindow,
};
use crate::mode::Mode;
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use terminalhud_core::Command;
static HOST_STARTED: AtomicBool = AtomicBool::new(false);
pub fn run_host(options: HostOptions) -> io::Result<()> {
    if HOST_STARTED.swap(true, Ordering::Relaxed) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "one GUI host per process",
        ));
    }
    options.validate()?;
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
    let controller = RuntimeController::new(&options)?;
    let production = options.mode == Mode::Real && options.development.is_none();
    let context = Box::new(HostContext::new(
        queue,
        controller,
        options.mode,
        None,
        production,
    ));
    let window = HostWindow::create(&context)?;
    context.window.set(window.0);
    context.menu_probe.set(options.menu_probe);
    let ui = UiSender::new(sender, window.0 as usize);
    let hotkey_messages = if options.mode == Mode::Real {
        Some(super::hotkey_messages::HotkeyMessages::install(ui.clone())?)
    } else {
        None
    };
    if options.mode == Mode::Real && options.development.is_some() {
        let (keys, error) = hotkeys::register();
        context.hotkey_error.set(error);
        *context.hotkeys.borrow_mut() = Some(keys);
    } else if options.mode == Mode::Real {
        *context.hotkeys.borrow_mut() = Some(hotkeys::Registrations::new());
    }
    if !options.headless {
        *context.tray.borrow_mut() = Some(tray_for_mode(
            ui.clone(),
            options.mode,
            context.hotkey_error.get(),
        )?);
    }
    window.configure_timers(&options)?;
    server.start(identity, ui.clone(), options.mode)?;
    if options.mode == Mode::Real
        && (options.development.is_some() || context.initialize_configuration())
    {
        context.initialize_real();
    }
    // Readiness is serviced only after the UI/tray/controller has been initialized.
    let run_result = super::message_loop::run();
    if !context.quitting.get()
        && let Err(error) = context.controller.borrow_mut().execute(Command::Quit)
    {
        eprintln!("TerminalHUD emergency restoration failed: {error}");
    }
    // Workers are cancelled/joined before removing their wake target. The server
    // retains its first-instance pipe handle until after the icon/window disappear.
    drop(context.hotkeys.borrow_mut().take());
    drop(hotkey_messages);
    ui.stop();
    server.stop();
    drop(context.tray.borrow_mut().take());
    drop(window);
    drop(server);
    let context_error = context.error.borrow_mut().take();
    run_result.and_then(|()| context_error.map_or(Ok(()), Err))
}
