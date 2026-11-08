//! Identity predicates shared by setup and thread-owned fixture cleanup.
#[derive(Debug)]
pub(crate) struct Identity {
    pub(crate) process: u32,
    pub(crate) thread: u32,
    pub(crate) class: Vec<u16>,
}
impl Identity {
    pub(crate) fn matches(&self, process: u32, thread: u32, class: &[u16]) -> bool {
        process == self.process && thread == self.thread && class == self.class
    }
}
