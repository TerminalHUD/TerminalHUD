//! An owned primary-monitor focus sink; never a normal user window.
mod identity;
mod messages;
mod run;
mod window;
use super::Result;
use identity::Identity;
use std::{
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
        mpsc,
    },
    thread::JoinHandle,
};
use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub(super) struct Underlay {
    hwnd: isize,
    clicks: Arc<AtomicU32>,
    identity: Identity,
    stop: mpsc::Sender<()>,
    thread: Option<JoinHandle<()>>,
}
impl Underlay {
    pub(super) fn start() -> Result<Self> {
        window::start()
    }
    pub(super) fn hwnd(&self) -> HWND {
        self.hwnd as HWND
    }
    pub(super) fn clicks(&self) -> u32 {
        self.clicks.load(Ordering::Relaxed)
    }
    pub(super) fn ensure_owned(&self) -> Result<()> {
        if self.thread.as_ref().is_none_or(JoinHandle::is_finished)
            || !is_owned(self.hwnd(), &self.identity)
        {
            return Err("owned sink identity/lifetime changed; refusing foreground request".into());
        }
        Ok(())
    }
}
fn is_owned(hwnd: HWND, expected: &Identity) -> bool {
    let mut process = 0;
    let mut class = [0u16; 256];
    unsafe {
        if IsWindow(hwnd) == 0 {
            return false;
        }
        let thread = GetWindowThreadProcessId(hwnd, &mut process);
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
        len > 0 && expected.matches(process, thread, &class[..len as usize])
    }
}
impl Drop for Underlay {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            // No message is sent to a possibly stale/reused HWND. Its owning
            // thread validates identity and destroys its window on this signal.
            if !thread.is_finished() {
                let _ = self.stop.send(());
            }
            if thread.join().is_err() {
                eprintln!("PHASE sink-cleanup failed: owning thread panicked");
            }
        }
    }
}
