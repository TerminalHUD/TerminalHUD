#[path = "../../../../windows-terminal/proof/windows/active/identity.rs"]
pub(super) mod identity;
#[path = "../../../../windows-terminal/proof/windows/active/input.rs"]
pub(super) mod input;
#[path = "../../../../windows-terminal/proof/windows/active/input_recovery.rs"]
mod input_recovery;
#[path = "../../../../windows-terminal/proof/windows/active/z_order.rs"]
pub(super) mod z_order;
pub(super) mod launch {
    pub(in super::super) use super::super::target::Launched;
}
pub(super) fn ensure(target: &launch::Launched) -> super::Result<()> {
    identity::ensure(target)
}
pub(super) fn fullscreen(target: &launch::Launched) -> super::Result<()> {
    identity::fullscreen(target).map(|_| ())
}
pub(super) fn above_sink(
    target: &launch::Launched,
    sink: &super::underlay::Underlay,
) -> super::Result<()> {
    z_order::ensure(target, sink)
}
pub(super) fn click_sink(
    target: &launch::Launched,
    sink: &super::underlay::Underlay,
) -> super::Result<()> {
    input::click_sink(target, sink)
}
