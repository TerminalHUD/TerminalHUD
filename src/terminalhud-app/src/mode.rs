//! Runtime identity is explicit in command routing and every reply.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Prototype,
    Real,
}
impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Self::Prototype => "prototype",
            Self::Real => "real",
        }
    }
}
pub const REAL_INSTANCE: &str = "real-default";
