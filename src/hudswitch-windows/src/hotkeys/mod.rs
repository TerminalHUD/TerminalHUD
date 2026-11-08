use std::{marker::PhantomData, rc::Rc};

mod binding;
mod cleanup;
mod error;
mod ids;
mod registration;

pub use binding::HotkeyBinding;
pub use error::{HotkeyFailure, HotkeyRegistrationError};

/// Thread-queue hotkeys; the caller owns and pumps the registering thread.
/// IDs must not be registered/unregistered by raw Win32 callers while owned.
/// Use `unregister` to observe cleanup failures; Drop is best effort.
///
/// ```compile_fail
/// fn require_send<T: Send>() {}
/// require_send::<hudswitch_windows::Hotkeys>();
/// ```
/// ```compile_fail
/// fn require_sync<T: Sync>() {}
/// require_sync::<hudswitch_windows::Hotkeys>();
/// ```
#[derive(Debug)]
pub struct Hotkeys {
    bindings: Vec<HotkeyBinding>,
    _thread: PhantomData<Rc<()>>,
}

impl Hotkeys {
    /// Bindings still owned, including any whose explicit cleanup failed.
    pub fn bindings(&self) -> &[HotkeyBinding] {
        &self.bindings
    }
}
