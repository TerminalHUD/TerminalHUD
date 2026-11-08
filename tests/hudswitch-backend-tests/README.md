# HudSwitch backend tests

`windows_backend.rs` is registered by `hudswitch-windows`. Ordinary workspace
tests run safe noninteractive cases on Windows. The three explicit desktop cases
are ignored by default and run serially only when requested with `--ignored`.

See `docs/windows-backend.md` at the repository root for commands, actual-input
assertions, foreground-policy handling, cleanup boundaries, and evidence limits.
Real OS checks use purpose-built child windows, never a normal user application.

The separate `hotkeys` target tests thread-owned registration/cleanup and native
conflicts with temporary uncommon chords, without pressing keys or exercising
the selected product bindings. See [the API/evidence limits](../../docs/windows-hotkeys.md).
