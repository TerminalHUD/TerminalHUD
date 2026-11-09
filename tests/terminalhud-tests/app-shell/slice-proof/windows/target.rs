//! Reverify the application's receipt independently from its local diagnostic.
use super::{Result, client::Owner};
use std::{
    collections::HashSet,
    fs,
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
};
use terminalhud_windows_terminal::{Guid, LaunchIdentity, SessionProof, native};
use windows_sys::Win32::{
    Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT},
    System::Threading::*,
    UI::WindowsAndMessaging::*,
};
pub(super) struct Session {
    bridge: OwnedHandle,
}
impl Session {
    pub(super) fn alive(&self) -> bool {
        (unsafe { WaitForSingleObject(self.bridge.as_raw_handle(), 0) }) == WAIT_TIMEOUT
    }
}
pub(super) struct Launched {
    pub session: Session,
    pub hwnd: isize,
    pub pid: u32,
    pub tid: u32,
    pub owner: OwnedHandle,
    pub original_style: isize,
    pub original_visible: bool,
    pub shell_pid: u32,
    pub session_id: String,
    pub nonce: Guid,
}
fn field<'a>(line: &'a str, key: &str) -> Result<&'a str> {
    Ok(line
        .split_whitespace()
        .find_map(|field| field.strip_prefix(key))
        .ok_or("diagnostic identity field missing")?)
}
impl Launched {
    pub(super) fn read(
        app: &Owner,
        terminal: &Path,
        profile: &Guid,
        before: &HashSet<isize>,
    ) -> Result<Self> {
        app.ensure()?;
        let text = fs::read_to_string(&app.log)?;
        let line = text
            .lines()
            .rfind(|line| line.starts_with("TerminalHUD development acquired "))
            .ok_or("acquisition diagnostic missing")?;
        let directory = line
            .split_once("directory=")
            .ok_or("directory missing")?
            .1
            .split_once(" nonce=")
            .ok_or("directory terminator missing")?
            .0;
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(Path::new(directory).join("session.json"))?)?;
        let guid = |name| -> Result<Guid> {
            Ok(Guid::parse(
                value[name].as_str().ok_or("session identity missing")?,
            )?)
        };
        let proof = SessionProof {
            nonce: guid("nonce")?,
            profile: guid("profile")?,
            session: guid("session")?,
            shell_pid: u32::try_from(value["shellPid"].as_u64().ok_or("bridge PID missing")?)?,
        };
        if &proof.profile != profile {
            return Err("manifest/profile session mismatch".into());
        }
        let identity = LaunchIdentity {
            nonce: proof.nonce.clone(),
            profile: proof.profile.clone(),
        };
        let bridge = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                proof.shell_pid,
            )
        };
        if bridge.is_null() {
            return Err("cannot retain owned bridge".into());
        }
        let session = Session {
            bridge: unsafe { OwnedHandle::from_raw_handle(bridge) },
        };
        let verified = native::verify(&identity, &proof, before, terminal)?;
        let pid = verified.process_id();
        let (hwnd, owner) = verified.into_parts();
        let tid = unsafe { GetWindowThreadProcessId(hwnd as _, std::ptr::null_mut()) };
        if field(line, "hwnd=")?.parse::<isize>()? != hwnd
            || field(line, "owner_pid=")?.parse::<u32>()? != pid
            || field(line, "nonce=")? != proof.nonce.as_str()
            || field(line, "session=")? != proof.session.as_str()
            || field(line, "profile=")? != profile.as_str()
            || field(line, "bridge_pid=")?.parse::<u32>()? != proof.shell_pid
        {
            return Err("app diagnostic and independently verified session differ".into());
        }
        let result = Self {
            session,
            hwnd,
            pid,
            tid,
            owner,
            original_style: field(line, "original_exstyle=")?.parse()?,
            original_visible: field(line, "original_visible=")?.parse()?,
            shell_pid: proof.shell_pid,
            session_id: proof.session.to_string(),
            nonce: proof.nonce,
        };
        super::active::ensure(&result)?;
        println!(
            "APP_OWNED hwnd={hwnd} owner={pid} bridge={} session={}",
            result.shell_pid, result.session_id
        );
        Ok(result)
    }
    pub(super) fn retain(&self) -> Result<Self> {
        super::active::ensure(self)?;
        Ok(Self {
            session: Session {
                bridge: self.session.bridge.try_clone()?,
            },
            hwnd: self.hwnd,
            pid: self.pid,
            tid: self.tid,
            owner: self.owner.try_clone()?,
            original_style: self.original_style,
            original_visible: self.original_visible,
            shell_pid: self.shell_pid,
            session_id: self.session_id.clone(),
            nonce: self.nonce.clone(),
        })
    }
    pub(super) fn epoch(&self, generation: u8, profile: &Guid) -> Result<serde_json::Value> {
        super::active::ensure(self)?;
        let mut value = super::startup_identity::process_json(self.owner.as_raw_handle())?;
        value["generation"] = generation.into();
        value["hwnd"] = self.hwnd.into();
        value["tid"] = self.tid.into();
        value["nonce"] = self.nonce.to_string().into();
        value["profile"] = profile.to_string().into();
        value["session"] = self.session_id.clone().into();
        value["bridge"] =
            super::startup_identity::process_json(self.session.bridge.as_raw_handle())?;
        value["powershell"] = serde_json::Value::Null;
        Ok(value)
    }
    pub(super) fn closed(&self) -> bool {
        unsafe {
            IsWindow(self.hwnd as _) == 0
                && WaitForSingleObject(self.owner.as_raw_handle(), 0) == WAIT_OBJECT_0
                && WaitForSingleObject(self.session.bridge.as_raw_handle(), 0) == WAIT_OBJECT_0
        }
    }
    pub(super) fn restored(&self) -> Result<()> {
        super::active::ensure(self)?;
        let current = unsafe { GetWindowLongPtrW(self.hwnd as _, GWL_EXSTYLE) };
        let mask = (WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT) as isize;
        if current & mask != self.original_style & mask
            || (unsafe { IsWindowVisible(self.hwnd as _) } != 0) != self.original_visible
        {
            return Err("Quit failed original owned style/visibility assertion".into());
        }
        println!(
            "APP_QUIT original styles/visibility restored; same bridge/session alive; interactive input still requires a genuine user command"
        );
        Ok(())
    }
}
