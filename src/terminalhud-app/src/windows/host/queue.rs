use super::super::native::last;
use crate::protocol::Reply;
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};
use terminalhud_core::Command;
pub enum UiMessage {
    Request {
        command: Command,
        reply: Option<mpsc::SyncSender<Reply>>,
    },
    QuitReplyWritten,
}

#[derive(Clone)]
pub struct UiSender {
    queue: mpsc::SyncSender<UiMessage>,
    window: usize,
    alive: Arc<AtomicBool>,
}
impl UiSender {
    pub fn new(queue: mpsc::SyncSender<UiMessage>, window: usize) -> Self {
        Self {
            queue,
            window,
            alive: Arc::new(AtomicBool::new(true)),
        }
    }
    pub(crate) fn stop(&self) {
        self.alive.store(false, Ordering::Release);
    }

    pub fn send(&self, message: UiMessage) -> io::Result<()> {
        if !self.alive.load(Ordering::Acquire) {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "UI host stopped"));
        }

        self.queue.try_send(message).map_err(|_| {
            io::Error::new(io::ErrorKind::WouldBlock, "UI command queue full/stopped")
        })?;
        if unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                self.window as _,
                super::WM_WAKE,
                0,
                0,
            )
        } == 0
        {
            return Err(last("PostMessage UI wake"));
        }
        Ok(())
    }
}
