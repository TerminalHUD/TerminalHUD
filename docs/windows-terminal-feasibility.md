# Windows Terminal feasibility

**Environment.** Windows 11 build 10.0.26200.0; official portable Windows
Terminal 1.24.12741.0, matching the same-version fixture. Preview and installed
Stable fragment-cache recognition were not tested.

**Scope and safety.** Runs used an isolated nonce-owned portable fixture and
synthetic settings, verifying exact HWND/process ancestry, executable, profile,
and session before control. Only the owned target was observed or controlled.
Runs ended with zero owned processes; the unrelated Terminal witness, user
settings, and global fragments remained unchanged. No physical input, user
settings changes, or unsupported launch bypass was used.

**Confirmed findings.** Fresh HUDs start Active per [ADR 0009](decisions/0009-active-hud-creation.md);
Passive and Hidden remain transitions. Bindings are unchanged. The portable
target activated with both `SW_SHOWNOACTIVATE` and `SW_SHOWNA`; fullscreen was
not required to reproduce activation. The cause is unknown. An earlier setup
timeout was inconclusive.

The exact-owned normal Active run passed launch/session ownership, fullscreen
bounds, topmost, focus, 35% background, and opaque-white text (1,562 qualifying
samples). Native Passive state and exact saved-foreground checks passed despite
a retained `FocusRefused` warning. Passive background reached 0%; completed
captures had zero white samples. A visual observation reported white appearing
briefly then disappearing. Frame-refresh and post-failure Layered-off controls
did not restore white. The cause is unproven; the initial style/lifecycle and
0% boundary remain possible factors.

**Acceptance boundary.** The captured post-failure controls do not establish the
cause or a fix. A private paired 0%/1% diagnostic is not a valid comparison
because it followed a failed pre-receipt wait. The 1% setting is unapproved and
does not satisfy 0% acceptance. Physical click routing, remaining Hidden/session
loops, and the normal successful-Quit/session-preservation phase have not run.
A direct focus warning alone does not prove whether a particular run took the
hide/show branch. The corrected proof acquisition allows normal backend
activation; resolution of the earlier timeout has not been empirically
established. Complete profile cloning is also unverified.

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
