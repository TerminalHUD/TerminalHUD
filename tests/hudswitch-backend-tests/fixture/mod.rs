use super::*;

mod input;
mod process;
mod window;

pub(crate) struct Fixture {
    child: Child,
    hwnd: isize,
}

pub(crate) fn wait(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !condition() {
        assert!(Instant::now() < deadline, "desktop observation timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub(crate) fn layered_attributes(hwnd: HWND) -> (u32, u8, u32) {
    let (mut color, mut alpha, mut flags) = (0, 0, 0);
    assert_ne!(
        unsafe { GetLayeredWindowAttributes(hwnd, &mut color, &mut alpha, &mut flags) },
        0
    );
    (color, alpha, flags)
}

pub(crate) fn center(hwnd: HWND) -> POINT {
    let mut rectangle = unsafe { std::mem::zeroed() };
    assert_ne!(unsafe { GetWindowRect(hwnd, &mut rectangle) }, 0);
    POINT {
        x: (rectangle.left + rectangle.right) / 2,
        y: (rectangle.top + rectangle.bottom) / 2,
    }
}

pub(crate) use input::click_owned;
