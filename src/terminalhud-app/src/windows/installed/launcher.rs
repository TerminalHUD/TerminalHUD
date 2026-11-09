//! Launch exactly one fresh named window in the installed Terminal host.
use super::Shared;
use crate::windows::development::{handshake, nonce, temporary_directory};
use hudswitch_windows::{Error, LaunchedWindow, Launcher, RecoveryClaim};
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

pub(in crate::windows) struct InstalledLauncher {
    shared: Shared,
    session: Option<(native::LauncherProcess, OwnedHandle, String)>,
}
impl InstalledLauncher {
    pub fn new(shared: Shared) -> Self {
        Self {
            shared,
            session: None,
        }
    }
    fn acquire(&mut self, intent: VisibleState) -> io::Result<LaunchedWindow> {
        if intent != VisibleState::Active {
            return Err(io::Error::other(
                "installed Terminal requires Active creation",
            ));
        }
        let settings = self.shared.refresh().map_err(io::Error::other)?;
        let identity = LaunchIdentity {
            nonce: nonce::fresh()?,
            profile: settings.profile,
        };
        let directory =
            temporary_directory()?.join(format!("TerminalHUD-session-{}", identity.nonce.bare()));
        fs::create_dir(&directory)?;
        fs::write(directory.join("nonce"), identity.nonce.as_str())?;
        fs::write(directory.join("commandline"), &settings.commandline)?;
        fs::write(
            directory.join("starting-directory"),
            settings.starting_directory.as_deref().unwrap_or(""),
        )?;
        let before: HashSet<_> = native::windows()
            .map_err(io::Error::other)?
            .iter()
            .map(|window| window.handle)
            .collect();
        let plan = LaunchPlan {
            identity: identity.clone(),
            intent,
            primary_origin: (0, 0),
        };
        let bridge = std::env::current_exe()?.with_file_name("terminalhud.exe");
        let command = vec![
            bridge.to_string_lossy().into_owned(),
            "--installed-session-bridge".into(),
            directory.to_string_lossy().into_owned(),
            identity.nonce.to_string(),
            identity.profile.to_string(),
        ];
        let launcher = native::start_with_command(&plan, &settings.terminal, &directory, &command)
            .map_err(io::Error::other)?;
        let deadline = Instant::now() + Duration::from_secs(8);
        let (proof, bridge) = loop {
            match handshake::read(&directory, &identity) {
                Ok(result) => break result,
                Err(error) if Instant::now() >= deadline => {
                    return Err(io::Error::other(format!(
                        "Terminal did not confirm the selected HUD profile; reload its settings and retry; any launched window remains unmanaged: {error}"
                    )));
                }
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        };
        let verified = loop {
            if unsafe { WaitForSingleObject(bridge.as_raw_handle(), 0) } != WAIT_TIMEOUT {
                return Err(io::Error::other(
                    "session bridge exited during window verification",
                ));
            }
            match native::verify(&identity, &proof, &before, &settings.terminal) {
                Ok(window) => break window,
                Err(error) if Instant::now() >= deadline => {
                    return Err(io::Error::other(format!(
                        "Terminal window could not be uniquely verified; it remains unmanaged: {error}"
                    )));
                }
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        };
        let (hwnd, owner) = verified.into_parts();
        if unsafe { WaitForSingleObject(bridge.as_raw_handle(), 0) } != WAIT_TIMEOUT {
            return Err(io::Error::other("session bridge exited before enrollment"));
        }
        // Adapter verification binds the nonce/session to this exact fresh HWND
        // and retains the actual owner process, which may host other windows.
        let receipt = unsafe { LaunchedWindow::from_verified_launch(hwnd as _, owner) }
            .map_err(io::Error::other)?;
        self.session = Some((launcher, bridge, identity.nonce.bare().replace('-', "")));
        Ok(receipt)
    }
}
impl Launcher for InstalledLauncher {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error> {
        self.acquire(intent).map_err(Error::Launch)
    }
    fn enroll(&mut self, claim: RecoveryClaim) -> Result<bool, Error> {
        let (_, bridge, instance) = self
            .session
            .as_ref()
            .ok_or(Error::Unsupported("missing recovery bridge"))?;
        super::super::recovery::enroll(instance, bridge, claim).map_err(Error::Launch)?;
        Ok(true)
    }
}
