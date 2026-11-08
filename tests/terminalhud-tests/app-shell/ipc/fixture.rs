use std::{
    process::{Child, Command as ProcessCommand, Stdio},
    thread,
    time::{Duration, Instant},
};
use terminalhud_app::{protocol, windows::ipc};
use terminalhud_core::Command;
pub(super) struct Primary {
    child: Child,
    instance: String,
}
impl Primary {
    pub(super) fn instance(&self) -> &str {
        &self.instance
    }
    pub(super) fn pid(&self) -> u32 {
        self.child.id()
    }
    pub(super) fn start() -> Self {
        Self::spawn(true)
    }
    pub(super) fn start_visible() -> Self {
        Self::spawn(false)
    }
    fn spawn(headless: bool) -> Self {
        let instance = format!("test-{:x}-{:x}", std::process::id(), unique() as u64);
        let mut command = ProcessCommand::new(env!("CARGO_BIN_EXE_terminalhud-shell"));
        command.args([
            "--prototype",
            "--instance",
            &instance,
            "--exit-after-ms",
            "30000",
        ]);
        if headless {
            command.arg("--headless");
        } else {
            command.arg("--menu-probe");
        }
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut primary = Self { child, instance };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Ok(reply) = ipc::call(&primary.instance, Command::Status) {
                assert_eq!(
                    reply.process_id,
                    primary.child.id(),
                    "endpoint is not our launched fixture"
                );
                break;
            }
            assert!(
                primary.child.try_wait().unwrap().is_none(),
                "primary exited during startup"
            );
            assert!(Instant::now() < deadline, "primary not ready");
            thread::sleep(Duration::from_millis(20));
        }
        primary
    }
    pub(super) fn call(&self, command: Command) -> protocol::Reply {
        ipc::call(&self.instance, command).unwrap()
    }
    pub(super) fn wait(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                assert!(status.success());
                return;
            }
            assert!(Instant::now() < deadline, "quit did not stop primary");
            thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Primary {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        if ipc::call(&self.instance, Command::Status)
            .is_ok_and(|reply| reply.process_id == self.child.id())
        {
            let _ = ipc::call(&self.instance, Command::Quit);
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn unique() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}
