//! Own screen/memory DCs and a selected top-down DIB through synchronized copying.
use super::{super::Result, render_pixels::Pixels, render_region::Region};
use std::{ffi::c_void, io, ptr, time::Instant};
use windows_sys::Win32::{Foundation::GetLastError, Graphics::Gdi::*};
struct Capture {
    screen: HDC,
    memory: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    bits: *mut c_void,
}
fn failure(operation: &str) -> Box<dyn std::error::Error> {
    let code = unsafe { GetLastError() };
    io::Error::other(format!("owned capture {operation} failed; win32={code}")).into()
}
pub(super) fn budget(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        return Err("capture censored: sample deadline exhausted".into());
    }
    Ok(())
}
pub(super) fn read(region: Region, deadline: Instant) -> Result<Pixels> {
    budget(deadline)?;
    let capture = Capture::new(region)?;
    budget(deadline)?;
    if unsafe {
        BitBlt(
            capture.memory,
            0,
            0,
            region.width() as i32,
            region.height() as i32,
            capture.screen,
            region.left(),
            region.top(),
            SRCCOPY | CAPTUREBLT,
        )
    } == 0
    {
        return Err(failure("BitBlt(SRCCOPY|CAPTUREBLT)"));
    }
    if unsafe { GdiFlush() } == 0 {
        return Err(failure("GdiFlush(batch)"));
    }
    budget(deadline)?;
    let stride = region
        .width()
        .checked_mul(4)
        .ok_or("capture stride overflow")?;
    let size = stride
        .checked_mul(region.height())
        .ok_or("capture size overflow")?;
    // GdiFlush completed writes; private bitmap/DC ownership remains live through
    // this bounded copy. No Rust reference to DIB memory survives this function.
    let bytes = unsafe { std::slice::from_raw_parts(capture.bits.cast::<u8>(), size) }.to_vec();
    budget(deadline)?;
    Ok(Pixels::new(region, stride, bytes)?)
}
impl Capture {
    fn new(region: Region) -> Result<Self> {
        let mut value = Self {
            screen: ptr::null_mut(),
            memory: ptr::null_mut(),
            bitmap: ptr::null_mut(),
            previous: ptr::null_mut(),
            bits: ptr::null_mut(),
        };
        value.screen = unsafe { GetDC(ptr::null_mut()) };
        if value.screen.is_null() {
            return Err(failure("GetDC"));
        }
        value.memory = unsafe { CreateCompatibleDC(value.screen) };
        if value.memory.is_null() {
            return Err(failure("CreateCompatibleDC"));
        }
        let mut info: BITMAPINFO = unsafe { std::mem::zeroed() };
        info.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: region.width() as i32,
            biHeight: -(region.height() as i32),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..unsafe { std::mem::zeroed() }
        };
        value.bitmap = unsafe {
            CreateDIBSection(
                value.screen,
                &info,
                DIB_RGB_COLORS,
                &mut value.bits,
                ptr::null_mut(),
                0,
            )
        };
        if value.bitmap.is_null() || value.bits.is_null() {
            return Err(failure("CreateDIBSection"));
        }
        let previous = unsafe { SelectObject(value.memory, value.bitmap) };
        if previous.is_null() || previous == GDI_ERROR as HGDIOBJ {
            return Err(failure("SelectObject"));
        }
        value.previous = previous;
        Ok(value)
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        unsafe {
            if !self.previous.is_null() {
                let restored = SelectObject(self.memory, self.previous);
                if restored.is_null() || restored == GDI_ERROR as HGDIOBJ {
                    eprintln!(
                        "CAPTURE_CLEANUP original bitmap restore failed; releasing only owned memory DC before bitmap deletion"
                    );
                    if DeleteDC(self.memory) == 0 {
                        eprintln!("CAPTURE_CLEANUP DeleteDC failed");
                    }
                    self.memory = ptr::null_mut();
                }
            }
            if !self.bitmap.is_null() && DeleteObject(self.bitmap) == 0 {
                eprintln!("CAPTURE_CLEANUP DeleteObject failed");
            }
            if !self.memory.is_null() && DeleteDC(self.memory) == 0 {
                eprintln!("CAPTURE_CLEANUP DeleteDC failed");
            }
            if !self.screen.is_null() && ReleaseDC(ptr::null_mut(), self.screen) == 0 {
                eprintln!("CAPTURE_CLEANUP ReleaseDC failed");
            }
        }
    }
}
