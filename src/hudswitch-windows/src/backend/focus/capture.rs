use super::super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn capture_foreground(&mut self) -> Result<Option<Focus>, Error> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_null() {
            return Ok(None);
        }
        let mut pid = 0;
        let tid = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                pid,
            )
        };
        if tid == 0 || handle.is_null() {
            return Err(last_error("OpenProcess foreground identity"));
        }
        Ok(Some(Focus {
            hwnd,
            pid,
            tid,
            process: unsafe { OwnedHandle::from_raw_handle(handle) },
        }))
    }
}
