use super::{UiMessage, WM_WAKE, context::HostContext};
use std::io;
use terminalhud_core::Command;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
impl HostContext {
    pub(super) fn dispatch(&self) {
        // This guard is checked on shared/interior-mutable state BEFORE a controller
        // borrow. Reentrant Win32 callbacks cannot form a second exclusive reference.
        if self.busy.replace(true) {
            self.deferred.set(true);
            return;
        }
        if self.refresh_requested.replace(false) && !self.quitting.get() {
            *self.observation_error.borrow_mut() = self.controller.borrow_mut().refresh().err();
        }
        if self.close_requested.replace(false) {
            let outcome = self.controller.borrow_mut().execute(Command::Quit);
            match outcome {
                Ok(outcome) if outcome.should_exit => unsafe {
                    EndMenu();
                    PostQuitMessage(0);
                },
                Err(error) => {
                    *self.error.borrow_mut() =
                        Some(io::Error::other(format!("quit failed: {error:?}")));
                }
                _ => {}
            }
        }
        while let Ok(event) = self.queue.try_recv() {
            match event {
                UiMessage::QuitReplyWritten => unsafe {
                    EndMenu();
                    PostQuitMessage(0);
                },
                UiMessage::Request { command, reply } => self.command(command, reply),
            }
        }
        self.busy.set(false);
        if self.deferred.replace(false) {
            unsafe {
                PostMessageW(self.window.get(), WM_WAKE, 0, 0);
            }
        }
    }
}
