use super::{Result, session::Session};
use hudswitch_core::{Controller, Event, Outcome, VisibleState};
use hudswitch_windows::{Error, LaunchedWindow, Launcher, WindowsBackend};
use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

struct Receipt(Option<LaunchedWindow>);
impl Launcher for Receipt {
    fn launch(&mut self, intent: VisibleState) -> std::result::Result<LaunchedWindow, Error> {
        if intent != VisibleState::Passive {
            return Err(Error::InvalidTarget);
        }
        self.0.take().ok_or(Error::InvalidTarget)
    }
}

pub(super) struct Managed {
    controller: Controller<WindowsBackend<Receipt>>,
    pub hwnd: isize,
    pub session: Session,
    original_bits: u32,
}
const OWNED: u32 = WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT;
impl Managed {
    pub(super) fn new(receipt: LaunchedWindow, hwnd: isize, session: Session) -> Result<Self> {
        let original_bits = unsafe { GetWindowLongPtrW(hwnd as HWND, GWL_EXSTYLE) } as u32 & OWNED;
        let mut managed = Self {
            controller: Controller::new(WindowsBackend::new(Receipt(Some(receipt)))?),
            hwnd,
            session,
            original_bits,
        };
        managed.dispatch(Event::VisibilityToggle)?;
        Ok(managed)
    }
    pub(super) fn dispatch(&mut self, event: Event) -> Result<Outcome<Error>> {
        self.controller
            .dispatch(event)
            .map_err(|error| format!("transition {event:?} failed: {error:?}").into())
    }
    pub(super) fn state(&self) -> Option<hudswitch_core::State> {
        self.controller.status().state
    }
    pub(super) fn quit(&mut self) -> Result<()> {
        self.dispatch(Event::Quit)?;
        let bits = unsafe { GetWindowLongPtrW(self.hwnd as HWND, GWL_EXSTYLE) } as u32 & OWNED;
        if bits != self.original_bits || unsafe { IsWindow(self.hwnd as HWND) } == 0 {
            return Err("quit changed owned style bits or closed the session".into());
        }
        Ok(())
    }
}
impl Drop for Managed {
    fn drop(&mut self) {
        // Preserve the controller/snapshot for bounded restore retries before the
        // explicit fixture-session cleanup closes only our dedicated test shell.
        for _ in 0..3 {
            if self.controller.dispatch(Event::Quit).is_ok() {
                return;
            }
        }
        eprintln!("RESTORE FAILED: fixture cleanup will close the dedicated test session");
    }
}
