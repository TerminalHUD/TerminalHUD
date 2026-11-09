use crate::observation::{Sample, Stage, Window};
use std::time::Instant;
use windows_sys::Win32::{
    Foundation::GetLastError,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
#[derive(Clone, Copy, Debug)]
pub(super) struct Reading {
    pub sample: Sample,
    pub foreground_first: isize,
    pub foreground_second: isize,
    pub owner_error: Option<u32>,
    pub foreground_pid: u32,
    pub foreground_tid: u32,
}
pub(super) fn read(origin: Instant, stage: Stage) -> Reading {
    let mut input = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    let input_ok = unsafe { GetLastInputInfo(&mut input) } != 0;
    let first = unsafe { GetForegroundWindow() };
    let mut pid = 0;
    let tid = unsafe { GetWindowThreadProcessId(first, &mut pid) };
    let owner_error = (tid == 0).then(|| unsafe { GetLastError() });
    let second = unsafe { GetForegroundWindow() };
    let foreground =
        (!first.is_null() && first == second && pid != 0 && tid != 0).then_some(Window {
            hwnd: first as isize,
            pid,
            tid,
        });
    Reading {
        sample: Sample {
            elapsed_us: origin.elapsed().as_micros().try_into().unwrap_or(u64::MAX),
            stage,
            input_tick: input_ok.then_some(input.dwTime),
            foreground,
        },
        foreground_first: first as isize,
        foreground_second: second as isize,
        owner_error,
        foreground_pid: pid,
        foreground_tid: tid,
    }
}
