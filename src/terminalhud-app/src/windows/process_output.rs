//! Finite file snapshots do not wait for EOF on inherited descendant pipe writers.
use std::{
    fs::{File, OpenOptions},
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
pub struct Capture {
    pub directory: PathBuf,
}
impl Capture {
    pub fn new() -> io::Result<Self> {
        let nonce = terminalhud_windows_terminal::native::fresh_guid()
            .map_err(|error| io::Error::other(error.to_string()))?;
        let directory = super::development::temporary_directory()?
            .join(format!("TerminalHUD-startup-capture-{}", nonce.bare()));
        std::fs::create_dir(&directory)?;
        Ok(Self { directory })
    }
    pub fn redirect(&self, command: &mut Command) -> io::Result<()> {
        let output = |name| {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.directory.join(name))
        };
        command
            .stdin(Stdio::null())
            .stdout(Stdio::from(output("stdout.txt")?))
            .stderr(Stdio::from(output("stderr.txt")?));
        Ok(())
    }
    pub fn read(&self) -> io::Result<(Vec<u8>, Vec<u8>)> {
        Ok((
            snapshot(&self.directory.join("stdout.txt"))?,
            snapshot(&self.directory.join("stderr.txt"))?,
        ))
    }
}
fn snapshot(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let length = file.metadata()?.len().min(65536);
    let mut bytes = Vec::new();
    file.take(length).read_to_end(&mut bytes)?;
    Ok(bytes)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        thread,
        time::{Duration, Instant},
    };
    #[test]
    fn a_long_lived_descendant_writer_does_not_delay_the_capture_snapshot() {
        let _serial = crate::windows::process_spawn::INHERITANCE_TEST
            .lock()
            .unwrap();
        let capture = Capture::new().unwrap();
        let powershell = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let script = r#"$info=[Diagnostics.ProcessStartInfo]::new();$info.FileName=(Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe');$info.Arguments='-NoProfile -NonInteractive -Command Start-Sleep -Milliseconds 1200';$info.UseShellExecute=$false;$info.CreateNoWindow=$true;$child=[Diagnostics.Process]::Start($info);[Console]::WriteLine($child.Id)"#;
        let mut command = Command::new(powershell);
        command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
        use std::os::windows::process::CommandExt;
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        capture.redirect(&mut command).unwrap();
        let mut parent = command.spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while parent.try_wait().unwrap().is_none() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        let start = Instant::now();
        let (out, _) = capture.read().unwrap();
        assert!(start.elapsed() < Duration::from_millis(500));
        let pid = String::from_utf8(out)
            .unwrap()
            .trim()
            .parse::<u32>()
            .unwrap();
        use windows_sys::Win32::{Foundation::*, System::Threading::*};
        let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        assert!(!process.is_null());
        let process =
            unsafe { crate::windows::native::Handle::from_raw(process, "descendant").unwrap() };
        assert_eq!(
            unsafe { WaitForSingleObject(process.raw(), 0) },
            WAIT_TIMEOUT,
            "descendant must still be alive when capture completes"
        );
        assert_eq!(
            unsafe { WaitForSingleObject(process.raw(), 5000) },
            WAIT_OBJECT_0
        );
        std::fs::remove_dir_all(capture.directory).unwrap();
    }
}
