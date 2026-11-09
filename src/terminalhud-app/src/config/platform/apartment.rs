use windows_sys::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
};

/// Balanced per-call COM initialization on the synchronous CLI thread.
pub(super) struct Apartment(std::marker::PhantomData<*mut ()>);
impl Apartment {
    pub fn enter() -> Result<Self, String> {
        // SAFETY: null reserved argument, supported STA flags on the current thread.
        let result = unsafe {
            CoInitializeEx(
                std::ptr::null(),
                (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
            )
        };
        if result < 0 {
            return Err(format!(
                "cannot initialize COM for configuration command: HRESULT {result:#010x}"
            ));
        }
        Ok(Self(std::marker::PhantomData))
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        // SAFETY: this guard is created only after successful CoInitializeEx.
        unsafe { CoUninitialize() };
    }
}
