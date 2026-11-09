//! Checked bounded screen geometry shared by capture and pure pixel access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Region {
    left: i32,
    top: i32,
    width: usize,
    height: usize,
}
impl Region {
    pub(crate) fn new(
        left: i32,
        top: i32,
        width: usize,
        height: usize,
    ) -> Result<Self, &'static str> {
        if width == 0 || height == 0 || width > 512 || height > 512 {
            return Err("capture region must be within 1..=512 pixels per axis");
        }
        left.checked_add(width as i32).ok_or("capture x overflow")?;
        top.checked_add(height as i32).ok_or("capture y overflow")?;
        Ok(Self {
            left,
            top,
            width,
            height,
        })
    }
    pub(crate) fn left(self) -> i32 {
        self.left
    }
    pub(crate) fn top(self) -> i32 {
        self.top
    }
    pub(crate) fn width(self) -> usize {
        self.width
    }
    pub(crate) fn height(self) -> usize {
        self.height
    }
    pub(crate) fn right(self) -> i32 {
        self.left + self.width as i32
    }
    pub(crate) fn bottom(self) -> i32 {
        self.top + self.height as i32
    }
    pub(crate) fn points(self) -> impl Iterator<Item = (i32, i32)> {
        (self.top..self.bottom())
            .step_by(8)
            .flat_map(move |y| (self.left..self.right()).step_by(8).map(move |x| (x, y)))
    }
}
