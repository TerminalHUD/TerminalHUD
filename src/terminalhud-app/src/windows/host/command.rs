use super::{TIMER_QUIT, context::HostContext};
use crate::protocol::{Reply, ReplyCode};
use std::sync::mpsc;
use terminalhud_core::Command;
use windows_sys::Win32::{System::Threading::GetCurrentProcessId, UI::WindowsAndMessaging::*};
impl HostContext {
    pub(super) fn command(&self, command: Command, reply: Option<mpsc::SyncSender<Reply>>) {
        let menu = reply.is_none();
        if menu {
            self.menu_commands
                .set(self.menu_commands.get().saturating_add(1));
        }
        let mut warning = false;
        if !self.quitting.get() {
            *self.observation_error.borrow_mut() = self.controller.borrow_mut().refresh().err();
        }
        let code = if self.quitting.get() {
            ReplyCode::ShuttingDown
        } else if self.requires_configuration
            && !self.launch_ready.get()
            && matches!(
                command,
                Command::InteractionToggle | Command::VisibilityToggle
            )
        {
            ReplyCode::TransitionFailed
        } else {
            // Arm the fallback before accepting an IPC quit; no successful quit
            // can strand an event loop if the reply worker subsequently fails.
            let timer_ok = command != Command::Quit
                || menu
                || unsafe { SetTimer(self.window.get(), TIMER_QUIT, 3000, None) } != 0;
            if !timer_ok {
                ReplyCode::TransitionFailed
            } else {
                if command != Command::Status {
                    self.sequence.set(self.sequence.get().saturating_add(1));
                }
                let outcome = if command == Command::Status
                    && (self.observation_error.borrow().is_some()
                        || self.last_command_error.borrow().is_some()
                        || self.configuration_error.borrow().is_some())
                {
                    Err("native observation failed; status is unconfirmed or recovered".into())
                } else {
                    self.controller.borrow_mut().execute(command)
                };
                match outcome {
                    Ok(outcome) => {
                        warning = outcome.focus_restore_error.is_some();
                        if command != Command::Status {
                            self.last_command_error.borrow_mut().take();
                        }
                        self.quitting.set(outcome.should_exit);
                        ReplyCode::Ok
                    }
                    Err(error) => {
                        if command != Command::Status {
                            eprintln!("TerminalHUD command {command:?} failed: {error}");
                            *self.last_command_error.borrow_mut() = Some(error);
                        }
                        if command == Command::Quit {
                            unsafe {
                                KillTimer(self.window.get(), TIMER_QUIT);
                            }
                        }
                        ReplyCode::TransitionFailed
                    }
                }
            }
        };
        let response = self.reply(code, warning);
        if let Some(sender) = reply {
            let _ = sender.send(response);
        }
        if self.quitting.get() && menu {
            unsafe {
                EndMenu();
                PostQuitMessage(0);
            }
        }
    }
    pub(super) fn reply(&self, code: ReplyCode, warning: bool) -> Reply {
        Reply {
            code,
            mode: self.mode,
            hotkey_error: self.hotkey_error.get(),
            status: self.controller.borrow().status(),
            focus_warning: warning,
            sequence: self.sequence.get(),
            menu_commands: self.menu_commands.get(),
            process_id: unsafe { GetCurrentProcessId() },
            host_console: !unsafe { windows_sys::Win32::System::Console::GetConsoleWindow() }
                .is_null(),
        }
    }
}
