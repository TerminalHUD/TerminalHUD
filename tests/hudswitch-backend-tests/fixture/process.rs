use super::*;

impl Fixture {
    pub(crate) fn launch(passive: bool, layered: bool) -> Self {
        static DPI: std::sync::Once = std::sync::Once::new();
        DPI.call_once(|| assert_ne!(unsafe { SetProcessDPIAware() }, 0));
        let executable = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/test-window.exe");
        assert!(
            executable.is_file(),
            "first run cargo build -p hudswitch-windows --example test-window"
        );
        let mut command = Command::new(executable);
        if passive {
            command.arg("--passive");
        }
        if layered {
            command.arg("--layered");
        }
        let mut child = command.stdout(Stdio::piped()).spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let result = BufReader::new(stdout).read_line(&mut line);
            let _ = sender.send((result, line));
        });
        let mut fixture = Self { child, hwnd: 0 };
        let (result, line) = receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("fixture did not become ready");
        result.unwrap();
        let fields: Vec<_> = line.split_whitespace().collect();
        assert_eq!(fields.first(), Some(&"READY"), "{line}");
        assert_eq!(fields[1].parse::<u32>().unwrap(), fixture.child.id());
        fixture.hwnd = fields[2].parse().unwrap();
        fixture
    }
    pub(crate) fn close(&mut self) {
        if self.hwnd != 0 {
            let mut pid = 0;
            unsafe {
                GetWindowThreadProcessId(self.hwnd(), &mut pid);
            }
            if pid == self.child.id() {
                unsafe {
                    PostMessageW(self.hwnd(), WM_CLOSE, 0, 0);
                }
            }
            self.hwnd = 0;
        }
        let deadline = Instant::now() + Duration::from_secs(1);
        while self.child.try_wait().unwrap().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.close();
    }
}
