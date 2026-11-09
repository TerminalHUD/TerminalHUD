use super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn acquire_target(
        &mut self,
        intent: VisibleState,
    ) -> Result<Acquired<Target, Original>, Error> {
        let window = self.launcher.launch(intent)?;
        // A receipt may have been held while its original window exited.
        window.ensure_current()?;
        if intent == VisibleState::Passive && unsafe { GetForegroundWindow() } == window.hwnd {
            return Err(Error::Unsupported("launcher activated a Passive target"));
        }
        if unsafe { GetClassLongPtrW(window.hwnd, GCL_STYLE) } as u32 & (CS_OWNDC | CS_CLASSDC) != 0
        {
            return Err(Error::Unsupported(
                "window class cannot use layered click-through",
            ));
        }
        let original = Original {
            exstyle: exstyle(window.hwnd)?,
            visible: unsafe { IsWindowVisible(window.hwnd) } != 0,
        };
        let token = loop {
            let value = NEXT_TOKEN.load(Ordering::Relaxed);
            let next = value
                .checked_add(1)
                .filter(|value| *value <= u32::MAX as usize)
                .ok_or(Error::Unsupported("acquisition token exhausted"))?;
            if NEXT_TOKEN
                .compare_exchange_weak(value, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break value;
            }
        };
        let token = usize::try_from((u64::from(std::process::id()) << 32) | token as u64)
            .map_err(|_| Error::Unsupported("64-bit Windows target required"))?;
        // Revalidate after snapshot/token preparation, then reject an existing
        // owner immediately before the metadata write. This is not an atomic
        // window-lifetime lock; callers must serialize exclusive acquisition.
        window.ensure_current()?;
        if !unsafe { GetPropW(window.hwnd, self.atom.key()) }.is_null() {
            return Err(Error::AlreadyManaged);
        }
        if unsafe { SetPropW(window.hwnd, self.atom.key(), token as HANDLE) } == 0 {
            return Err(last_error("SetPropW (check privilege compatibility)"));
        }
        let claim = RecoveryClaim::new(&window, token, &original);
        let recovery_enrolled = match self.launcher.enroll(claim) {
            Ok(value) => value,
            Err(error) => {
                if unsafe { GetPropW(window.hwnd, self.atom.key()) } as usize == token {
                    unsafe {
                        RemovePropW(window.hwnd, self.atom.key());
                    }
                }
                return Err(error);
            }
        };
        let target = Target {
            window,
            atom: self.atom.clone(),
            token,
            original_layered: original.exstyle & WS_EX_LAYERED != 0,
            needs_layer_init: Cell::new(false),
            changed_style: Cell::new(0),
            changed_topmost: Cell::new(false),
            changed_visibility: Cell::new(false),
            recovery_enrolled,
        };
        target.ensure()?;
        Ok(Acquired { target, original })
    }
}
