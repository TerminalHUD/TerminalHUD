use crate::windows::native::{Handle, last};
use std::io;
use windows_sys::Win32::{Foundation::*, System::Threading::*};
pub(crate) struct Process {
    pub handle: Handle,
    pub pid: u32,
}
impl Process {
    pub(crate) fn try_wait(&self) -> io::Result<Option<u32>> {
        match unsafe { WaitForSingleObject(self.handle.raw(), 0) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0;
                if unsafe { GetExitCodeProcess(self.handle.raw(), &mut code) } == 0 {
                    return Err(last("GetExitCodeProcess GUI"));
                }
                Ok(Some(code))
            }
            _ => Err(last("WaitForSingleObject GUI")),
        }
    }
    pub(crate) fn terminate_prototype(&self) -> io::Result<()> {
        if unsafe { TerminateProcess(self.handle.raw(), 2) } == 0 {
            return Err(last("TerminateProcess owned prototype startup"));
        }
        if unsafe { WaitForSingleObject(self.handle.raw(), 5000) } != WAIT_OBJECT_0 {
            return Err(last("Wait terminated prototype"));
        }
        Ok(())
    }
}
