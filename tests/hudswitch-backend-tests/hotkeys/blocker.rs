use super::super::*;
use std::{sync::mpsc, thread};

/// Keeps a deliberately conflicting, test-owned chord alive on another thread.
pub(super) struct Blocker {
    stop: Option<mpsc::Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Blocker {
    pub(super) fn start(binding: HotkeyBinding) -> Self {
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            let mut owned = match Hotkeys::register(&[binding]) {
                Ok(owned) => owned,
                Err(error) => {
                    ready_tx.send(Err(error.to_string())).unwrap();
                    return;
                }
            };
            ready_tx.send(Ok(())).unwrap();
            let _ = stop_rx.recv();
            owned.unregister().unwrap();
        });
        let blocker = Self {
            stop: Some(stop_tx),
            thread: Some(thread),
        };
        ready_rx.recv().unwrap().expect("test chord unavailable");
        blocker
    }
}

impl Drop for Blocker {
    fn drop(&mut self) {
        self.stop.take();
        self.thread.take().unwrap().join().unwrap();
    }
}
