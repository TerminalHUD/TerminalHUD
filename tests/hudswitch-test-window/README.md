# Generic test window

This directory owns the generic window harness used to validate a HudSwitch
backend independently of terminal applications. The actual window is implemented
with the backend proof.

`toolchain_probe.rs` is a console-only Cargo example registered by
`hudswitch-windows`. It prints its compiled OS, architecture, and process ID to
prove the native build/run path. It does not create a window or establish GUI,
focus, hotkey, or click-through correctness.
