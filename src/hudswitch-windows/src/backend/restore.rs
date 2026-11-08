use super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn restore_target(
        &mut self,
        target: &Target,
        original: &Original,
    ) -> Result<(), Error> {
        target.ensure()?;
        let changed = target.changed_style.get();
        set_style(target, changed, original.exstyle)?;
        let topmost = target
            .changed_topmost
            .get()
            .then_some(original.exstyle & WS_EX_TOPMOST != 0);
        let visible = target.changed_visibility.get().then_some(original.visible);
        position(target, topmost, visible)?;
        if exstyle(target.window.hwnd)? & changed != original.exstyle & changed {
            return Err(Error::Timeout("original style restoration"));
        }
        target.ensure()?;
        target.remove_own_marker()
    }
}
