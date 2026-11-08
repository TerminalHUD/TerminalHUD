# TerminalHUD tests

Product orchestration and adapter suites live here. Register real suites with
`[[test]]` targets in their owning product crate. Keep generic HudSwitch behavior
in its own core/backend tests; product suites may compose those components.
