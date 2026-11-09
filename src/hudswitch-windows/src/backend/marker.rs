use super::*;

impl IdentityAtom {
    pub(super) fn key(&self) -> *const u16 {
        self.0 as usize as *const u16
    }
}
impl Drop for IdentityAtom {
    fn drop(&mut self) {
        // Integer-atom properties do not allocate an atom per target acquisition.
        unsafe {
            GlobalDeleteAtom(self.0);
        }
    }
}

impl Target {
    pub fn window_handle(&self) -> HWND {
        self.window.hwnd
    }
    pub(super) fn valid(&self) -> bool {
        self.window.has_current_identity()
            && unsafe { GetPropW(self.window.hwnd, self.atom.key()) } as usize == self.token
    }

    pub(super) fn remove_own_marker(&self) -> Result<(), Error> {
        // Cleanup must also work after identity validation fails. Never remove
        // a marker whose exact token is not ours, even on a recycled HWND.
        if unsafe { GetPropW(self.window.hwnd, self.atom.key()) } as usize != self.token {
            return Err(Error::InvalidTarget);
        }
        if unsafe { RemovePropW(self.window.hwnd, self.atom.key()) } as usize != self.token {
            return Err(last_error("RemovePropW acquisition identity"));
        }
        Ok(())
    }
    pub(super) fn ensure(&self) -> Result<(), Error> {
        if self.valid() {
            Ok(())
        } else {
            Err(Error::InvalidTarget)
        }
    }
}
impl Drop for Target {
    fn drop(&mut self) {
        if !self.recovery_enrolled {
            let _ = self.remove_own_marker();
        }
    }
}
