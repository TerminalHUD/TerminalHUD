# Generic test window

`main.rs` is the `hudswitch-windows` Cargo example `test-window`. It creates an
ordinary top-level Win32 window, owns its native message loop, and announces its
PID/HWND on stdout for the dedicated launcher. `--passive` selects nonactivating
creation; `--layered` gives the preservation case existing alpha/color-key state.
The process closes normally on WM_CLOSE. It contains no terminal behavior.

The backend test runner uses only recorded fixture children. Test-only WM_APP
messages expose delivered mouse counts and apply an unrelated style bit so
validation can distinguish actual input routing and owned-style restoration.

`toolchain_probe.rs` remains the console-only example `toolchain-probe`. It proves
native build/run identity and creates no window. It is not GUI validation.
