# HudSwitch backend tests

Backend integration suites live here and use the generic test window rather than
a terminal. Register Windows suites with `[[test]]` targets in
`src/hudswitch-windows/Cargo.toml`, gating platform-specific source appropriately.
Tests requiring an interactive desktop must be distinguished from unattended CI.
