use super::{
    super::native::{last, wide},
    Identity,
};
use std::{io, mem, ptr};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{Authorization::*, *},
};
pub struct Descriptor(*mut core::ffi::c_void);
impl Descriptor {
    pub fn for_user(identity: &Identity) -> io::Result<Self> {
        // Protected DACL grants this SID alone; do not use the permissive default.
        let sddl = wide(&format!("O:{0}G:{0}D:P(A;;GA;;;{0})", identity.sid));
        let mut raw = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut raw,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(last("ConvertStringSecurityDescriptor"));
        }
        Ok(Self(raw))
    }
    pub fn attributes(&self) -> SECURITY_ATTRIBUTES {
        SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: self.0,
            bInheritHandle: 0,
        }
    }
}
impl Drop for Descriptor {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
