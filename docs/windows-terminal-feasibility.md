# Windows Terminal feasibility

**Environment.** Windows 11 build 10.0.26200.0; official portable Windows
Terminal 1.24.12741.0, matching the same-version fixture. Preview and installed
Stable fragment-cache recognition were not tested.

**Scope and safety.** Runs used an isolated nonce-owned portable fixture and
synthetic settings, verifying exact HWND/process ancestry, executable, profile,
and session before control. Mutations and pixel sampling were restricted to
owned fixtures; unrelated Terminal windows were inspected read-only as witnesses.
Runs ended with zero owned processes; user settings and global fragments remained
unchanged. The corrected full proof used one guarded physical click and no
keyboard input, user settings changes, or unsupported launch bypass.

**Confirmed findings.** Fresh HUDs start Active per [ADR 0009](decisions/0009-active-hud-creation.md);
Passive and Hidden remain transitions. Bindings are unchanged. The portable
target activated with both `SW_SHOWNOACTIVATE` and `SW_SHOWNA`; fullscreen was
not required to reproduce activation. The cause is unknown. An earlier setup
timeout was inconclusive.

The corrected full normal proof at
[`dd23d93`](https://github.com/TerminalHUD/TerminalHUD/commit/dd23d93f7dc445f814ac40efd1afa8fee9ff6dd0)
exited 0. Exact-owned launch/session, primary fullscreen, topmost and Active
focus passed. Active 35% background measured RGB (26,104,143); Passive 0%
measured (40,160,220), each with 1,562 opaque-white samples. One guarded physical
click reached the owned sink. Passive/Hidden/Passive and Active/Hidden/Active
preserved the exact session. Normal Quit restored owned styles/visibility and
left that session alive before explicit nonce cleanup. A Passive `FocusRefused`
API warning remained recorded while strict saved-foreground/native checks passed;
the actual release branch was not logged.

**Historical ambiguity.** Earlier zero-white captures and a visual report of
white briefly appearing then disappearing used an opaque topmost blue background
without relative-order proof. They cannot distinguish occlusion from renderer
loss. The corrected proof uses an ordinary non-topmost background and checks
actual target-above-background order around capture and click routing, without
repositioning the HUD to force a pass. This does not prove the sole historical
cause. Product/backend alpha initialization is unchanged.

**Remaining limits.** Results cover this dedicated portable version and synthetic
profile. Installed fragment reload, Preview and complete ordinary-profile cloning
remain unverified. This adapter proof does not establish the real application
shell/tray/IPC/selected-shortcut composition tracked in TER-11.

**Reproduction and raw records.** See the [Windows Terminal proof guide](windows-terminal-proof.md)
and [TER-9/TER-14 draft PR #9](https://github.com/TerminalHUD/TerminalHUD/pull/9).
Raw traces are preserved at immutable checkpoint
[`6f611dc`](https://github.com/TerminalHUD/TerminalHUD/commit/6f611dcc46ee353cebfdb38400ccc30d96f4676b):
[run 1](https://github.com/TerminalHUD/TerminalHUD/blob/6f611dcc46ee353cebfdb38400ccc30d96f4676b/tests/terminalhud-tests/windows-terminal/evidence/active-run1.txt),
[run 2](https://github.com/TerminalHUD/TerminalHUD/blob/6f611dcc46ee353cebfdb38400ccc30d96f4676b/tests/terminalhud-tests/windows-terminal/evidence/active-run2.txt),
[run 3](https://github.com/TerminalHUD/TerminalHUD/blob/6f611dcc46ee353cebfdb38400ccc30d96f4676b/tests/terminalhud-tests/windows-terminal/evidence/active-run3.txt),
[run 4](https://github.com/TerminalHUD/TerminalHUD/blob/6f611dcc46ee353cebfdb38400ccc30d96f4676b/tests/terminalhud-tests/windows-terminal/evidence/active-run4.txt).
The earlier [fullscreen trace](https://github.com/TerminalHUD/TerminalHUD/blob/dd1ee5d551352c69407ae349323d4ec179f2362e/tests/terminalhud-tests/windows-terminal/evidence/show8-phased.txt)
and [windowed control](https://github.com/TerminalHUD/TerminalHUD/blob/dd1ee5d551352c69407ae349323d4ec179f2362e/tests/terminalhud-tests/windows-terminal/evidence/windowed-show8.txt)
are also published.
