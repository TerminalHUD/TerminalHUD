use super::*;

impl<L: Launcher> WindowsBackend<L> {
    pub(in crate::backend) fn present_target(
        &mut self,
        target: &Target,
        presentation: Presentation,
    ) -> Result<(), Error> {
        target.ensure()?;
        if presentation == Presentation::Hidden {
            return position(target, None, Some(false));
        }
        let style = exstyle(target.window.hwnd)?;
        let mut mask = WS_EX_TRANSPARENT;
        let mut desired = 0;
        if presentation == Presentation::Passive {
            if target.original_layered && style & WS_EX_LAYERED == 0 {
                return Err(Error::Unsupported(
                    "target changed its renderer layering mode",
                ));
            }
            desired |= WS_EX_TRANSPARENT;
            if style & WS_EX_LAYERED == 0 {
                mask |= WS_EX_LAYERED;
                desired |= WS_EX_LAYERED;
                target.needs_layer_init.set(true);
            }
        } else if !target.original_layered && target.changed_style.get() & WS_EX_LAYERED != 0 {
            mask |= WS_EX_LAYERED;
        }
        set_style(target, mask, desired)?;
        if presentation == Presentation::Active {
            target.needs_layer_init.set(false);
        }
        if presentation == Presentation::Passive && target.needs_layer_init.get() {
            // Only initialize layering we added to an originally nonlayered window.
            // Existing layered/colorkey/per-pixel state is never rewritten.
            target.ensure()?;
            if unsafe { SetLayeredWindowAttributes(target.window.hwnd, 0, 255, LWA_ALPHA) } == 0 {
                return Err(last_error(
                    "SetLayeredWindowAttributes opaque initialization",
                ));
            }
            target.needs_layer_init.set(false);
        }
        position(target, Some(true), Some(true))
    }
}
