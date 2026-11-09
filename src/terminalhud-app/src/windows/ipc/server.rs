use super::super::{
    host::UiSender,
    native::Handle,
    security::{Descriptor, Identity},
};
use super::{IO_TIMEOUT, connection::handle_connection};
use crate::protocol::{REPLY_LEN, REQUEST_LEN};
use std::{
    io, ptr,
    sync::Arc,
    thread::{self, JoinHandle},
};
use windows_sys::Win32::{
    Foundation::*,
    Storage::FileSystem::*,
    System::{IO::CancelIoEx, Pipes::*, Threading::*},
};
/// Retained across every accept and until the UI is torn down.
pub struct Server {
    pub(super) pipe: Arc<Handle>,
    pub(super) stop: Arc<Handle>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    pub fn bind(identity: &Identity, instance: &str) -> io::Result<Self> {
        let name = identity.pipe_name(instance)?;
        let descriptor = Descriptor::for_user(identity)?;
        let attributes = descriptor.attributes();
        let pipe = unsafe {
            Handle::from_raw(
                CreateNamedPipeW(
                    name.as_ptr(),
                    PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                    PIPE_TYPE_MESSAGE
                        | PIPE_READMODE_MESSAGE
                        | PIPE_WAIT
                        | PIPE_REJECT_REMOTE_CLIENTS,
                    1,
                    REPLY_LEN as u32,
                    REQUEST_LEN as u32,
                    IO_TIMEOUT,
                    &attributes,
                ),
                "CreateNamedPipe",
            )?
        };
        let stop = unsafe {
            Handle::from_raw(
                CreateEventW(ptr::null(), 1, 0, ptr::null()),
                "CreateEvent stop",
            )?
        };
        Ok(Self {
            pipe: Arc::new(pipe),
            stop: Arc::new(stop),
            worker: None,
        })
    }
    pub fn start(
        &mut self,
        identity: Identity,
        ui: UiSender,
        mode: crate::mode::Mode,
    ) -> io::Result<()> {
        let pipe = self.pipe.clone();
        let stop = self.stop.clone();
        self.worker = Some(
            thread::Builder::new()
                .name("terminalhud-ipc".into())
                .spawn(move || {
                    while unsafe { WaitForSingleObject(stop.raw(), 0) } != WAIT_OBJECT_0 {
                        let result = handle_connection(&pipe, &stop, &identity, &ui, mode);
                        unsafe {
                            DisconnectNamedPipe(pipe.raw());
                        }
                        if matches!(result, Ok(true)) {
                            break;
                        }
                        // Errors affect this connection only. The first-instance handle stays owned.
                    }
                })?,
        );
        Ok(())
    }
    pub fn stop(&mut self) {
        unsafe {
            SetEvent(self.stop.raw());
            CancelIoEx(self.pipe.raw(), ptr::null());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}
