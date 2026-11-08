use super::{
    super::{observe, setup::Setup},
    session::ActiveSession,
};
use hudswitch_core::VisibleState;
use hudswitch_windows::{Error, LaunchedWindow, Launcher};
use std::{
    cell::RefCell,
    collections::HashSet,
    io,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};
use terminalhud_windows_terminal::{LaunchPlan, native};
pub(super) struct Launched {
    pub(super) session: ActiveSession,
    pub(super) hwnd: isize,
    pub(super) pid: u32,
    pub(super) tid: u32,
    pub(super) owner: std::os::windows::io::OwnedHandle,
    pub(super) original_style: isize,
    pub(super) original_visible: bool,
    pub(super) shell_pid: u32,
    pub(super) session_id: String,
}
pub(super) type Shared = Rc<RefCell<Option<Launched>>>;
pub(super) struct AdapterLauncher<'a> {
    setup: &'a Setup,
    shared: Shared,
}
impl<'a> AdapterLauncher<'a> {
    pub(super) fn new(setup: &'a Setup, shared: Shared) -> Self {
        Self { setup, shared }
    }
    fn acquire(&self, intent: VisibleState) -> super::super::Result<LaunchedWindow> {
        if intent != VisibleState::Active {
            return Err("acceptance launcher requires an explicit Active request".into());
        }
        let before: HashSet<_> = native::windows()?.iter().map(|w| w.handle).collect();
        let plan = LaunchPlan {
            identity: self.setup.identity.clone(),
            intent,
            primary_origin: (0, 0),
        };
        let launcher = native::start_with_command(
            &plan,
            &self.setup.terminal,
            &self.setup.task,
            &self.setup.command,
        )?;
        let mut session = ActiveSession::new(self.setup, launcher);
        let proof = session.handshake()?;
        let deadline = Instant::now() + Duration::from_secs(5);
        let verified = loop {
            match native::verify(&self.setup.identity, &proof, &before, &self.setup.terminal) {
                Ok(window) => break window,
                Err(error) if Instant::now() >= deadline => return Err(error.into()),
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        };
        let hwnd = verified.window_handle();
        let pid = verified.process_id();
        observe::wait(|| unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd as _) } != 0 && unsafe {windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow()} == hwnd as _)?;
        let tid = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(
                hwnd as _,
                std::ptr::null_mut(),
            )
        };
        let original_style = unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                hwnd as _,
                windows_sys::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
            )
        };
        let original_visible =
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd as _) } != 0;
        println!(
            "ACTIVE_IDENTITY launcher={} owner={pid} hwnd={hwnd} shell={} session={}",
            session.launcher.pid, proof.shell_pid, proof.session
        );
        let (raw, owner) = verified.into_parts();
        // Exact fresh HWND/owner image, retained creation ancestry and nonce/session
        // correlation above satisfy this explicit delegated-launch trust boundary.
        let retained_owner = owner.try_clone()?;
        let receipt = unsafe { LaunchedWindow::from_verified_launch(raw as _, owner) }?;
        *self.shared.borrow_mut() = Some(Launched {
            session,
            hwnd,
            pid,
            tid,
            owner: retained_owner,
            original_style,
            original_visible,
            shell_pid: proof.shell_pid,
            session_id: proof.session.to_string(),
        });
        Ok(receipt)
    }
}
impl Launcher for AdapterLauncher<'_> {
    fn launch(&mut self, intent: VisibleState) -> Result<LaunchedWindow, Error> {
        self.acquire(intent)
            .map_err(|error| Error::Launch(io::Error::other(error.to_string())))
    }
}
