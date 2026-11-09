use super::{Configuration, error, handshake, namespaces, nonce};
use hudswitch_windows::{Error, LaunchedWindow, Launcher};
use std::{
    collections::HashSet,
    fs, io,
    os::windows::io::{AsRawHandle, OwnedHandle},
    thread,
    time::{Duration, Instant},
};
use terminalhud_core::integration::VisibleState;
use terminalhud_windows_terminal::{LaunchIdentity, LaunchPlan, native};
use windows_sys::Win32::{Foundation::WAIT_TIMEOUT, System::Threading::WaitForSingleObject};
pub(in crate::windows) struct DevelopmentLauncher {
    config: Configuration,
    // Retain live bridge and launch process identities until next acquisition;
    // dropping either handle deliberately leaves the terminal session running.
    session: Option<(native::LauncherProcess, OwnedHandle)>,
}
impl DevelopmentLauncher {
    pub(in crate::windows) fn new(config: Configuration) -> Self {
        Self {
            config,
            session: None,
        }
    }
    fn acquire(&mut self, intent: VisibleState) -> io::Result<LaunchedWindow> {
        if intent != VisibleState::Active {
            return Err(error("development slice requires Active creation"));
        }
        self.config.validate()?;
        namespaces::check()?;
        let identity = LaunchIdentity {
            nonce: nonce::fresh()?,
            profile: self.config.profile.clone(),
        };
        let directory = super::temporary::directory()?
            .join(format!("TerminalHUD-session-{}", identity.nonce.bare()));
        fs::create_dir(&directory)?;
        fs::write(directory.join("nonce"), identity.nonce.as_str())?;
        let before: HashSet<_> = native::windows()
            .map_err(error)?
            .iter()
            .map(|window| window.handle)
            .collect();
        let plan = LaunchPlan {
            identity: identity.clone(),
            intent,
            primary_origin: (0, 0),
        };
        let command = vec![
            self.config.bridge.to_string_lossy().into_owned(),
            "--session-bridge".into(),
            directory.to_string_lossy().into_owned(),
            identity.nonce.to_string(),
            identity.profile.to_string(),
        ];
        let launcher =
            native::start_with_command(&plan, &self.config.terminal, &directory, &command)
                .map_err(error)?;
        let deadline = Instant::now() + Duration::from_secs(6);
        let (proof, bridge) = loop {
            match handshake::read(&directory, &identity) {
                Ok(proof) => break proof,
                Err(e) if Instant::now() >= deadline => {
                    return Err(error(format!(
                        "development handshake failed: {e}; any launched terminal remains unmanaged"
                    )));
                }
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        };
        let verified = loop {
            if unsafe { WaitForSingleObject(bridge.as_raw_handle(), 0) } != WAIT_TIMEOUT {
                return Err(error("development bridge exited before acquisition"));
            }
            match native::verify(&identity, &proof, &before, &self.config.terminal) {
                Ok(window) => break window,
                Err(e) if Instant::now() >= deadline => {
                    return Err(error(format!(
                        "development target verification failed: {e}; terminal remains unmanaged"
                    )));
                }
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        };
        let pid = verified.process_id();
        let (hwnd, owner) = verified.into_parts();
        if unsafe { WaitForSingleObject(bridge.as_raw_handle(), 0) } != WAIT_TIMEOUT {
            return Err(error("development bridge exited during verification"));
        }
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GWL_EXSTYLE, GetWindowLongPtrW, IsWindowVisible,
        };
        let style = unsafe { GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE) };
        let visible = unsafe { IsWindowVisible(hwnd as _) } != 0;
        eprintln!(
            "TerminalHUD development acquired directory={} nonce={} profile={} session={} bridge_pid={} owner_pid={pid} hwnd={hwnd} original_exstyle={style} original_visible={visible}",
            directory.display(),
            identity.nonce,
            identity.profile,
            proof.session,
            proof.shell_pid
        );
        // Exact fresh HWND, retained executable/creation ancestry and nonce/profile/
        // session proof satisfy the backend's delegated-launch receipt contract.
        let receipt =
            unsafe { LaunchedWindow::from_verified_launch(hwnd as _, owner) }.map_err(error)?;
        self.session = Some((launcher, bridge));
        Ok(receipt)
    }
}
impl Launcher for DevelopmentLauncher {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error> {
        self.acquire(intent).map_err(Error::Launch)
    }
}
