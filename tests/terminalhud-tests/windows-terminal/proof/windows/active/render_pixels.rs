//! Owned immutable top-down BGRA data; no borrowed native bitmap memory escapes.
use super::{render_evidence::Brightest, render_region::Region};
use std::time::Instant;
pub(crate) struct Pixels {
    region: Region,
    stride: usize,
    bytes: Vec<u8>,
}
#[derive(Debug)]
pub(crate) struct Summary {
    pub(crate) white: usize,
    pub(crate) brightest: Brightest,
}
impl Pixels {
    pub(crate) fn new(region: Region, stride: usize, bytes: Vec<u8>) -> Result<Self, &'static str> {
        let row = region.width().checked_mul(4).ok_or("pixel row overflow")?;
        let size = stride
            .checked_mul(region.height())
            .ok_or("pixel buffer overflow")?;
        if stride < row
            || stride > 2048
            || !stride.is_multiple_of(4)
            || size > 512 * 512 * 4
            || bytes.len() != size
        {
            return Err("invalid bounded BGRA stride/length");
        }
        Ok(Self {
            region,
            stride,
            bytes,
        })
    }
    pub(crate) fn rgb(&self, x: i32, y: i32) -> Result<[i32; 3], &'static str> {
        let column = x
            .checked_sub(self.region.left())
            .ok_or("pixel x overflow")?;
        let row = y.checked_sub(self.region.top()).ok_or("pixel y overflow")?;
        if column < 0
            || row < 0
            || column as usize >= self.region.width()
            || row as usize >= self.region.height()
        {
            return Err("pixel outside captured region");
        }
        let offset = (row as usize)
            .checked_mul(self.stride)
            .and_then(|v| v.checked_add(column as usize * 4))
            .ok_or("pixel offset overflow")?;
        let pixel = self
            .bytes
            .get(offset..offset + 3)
            .ok_or("pixel data truncated")?;
        Ok([
            i32::from(pixel[2]),
            i32::from(pixel[1]),
            i32::from(pixel[0]),
        ])
    }
    pub(crate) fn summarize(&self, deadline: Instant) -> Result<Summary, &'static str> {
        let mut summary = Summary {
            white: 0,
            brightest: Brightest {
                rgb: [0, 0, 0],
                point: (self.region.left(), self.region.top()),
            },
        };
        for (x, y) in self.region.points() {
            if Instant::now() >= deadline {
                return Err("pixel processing censored: sample deadline exhausted");
            }
            let rgb = self.rgb(x, y)?;
            summary.brightest.consider(rgb, (x, y));
            if rgb.into_iter().all(|value| value >= 245) {
                summary.white += 1;
            }
        }
        Ok(summary)
    }
}
