# Windows Terminal startup findings

TerminalHUD creates every fresh HUD Active, as accepted in
[ADR 0009](decisions/0009-active-hud-creation.md). Passive and Hidden remain
transitions for existing targets. Normal adapter APIs reject an explicit Passive
creation request before spawning; there is no experimental launch bypass.

## Measured startup behavior

On Windows 11 build 10.0.26200.0, the isolated official Windows Terminal
1.24.12741.0 fixture activated its exact verified target with both
`SW_SHOWNOACTIVATE(4)` and `SW_SHOWNA(8)`. The show8 windowed control also
activated, so fullscreen is not necessary to reproduce the behavior. The exact
internal cause and any fundamental Windows limitation were not established.
An earlier show8 setup timeout was inconclusive and is not activation evidence.

The phased fullscreen and windowed runs verified fresh HWND, executable,
retained owner ancestry and nonce/profile/session before classifying the
foreground trace. Only the owned target activated; no unrelated event
contaminated those observations. No backend, styles or input followed.
Owned fixtures exited and external settings/fragments/window witnesses stayed
unchanged. The original
[fullscreen trace](https://github.com/TerminalHUD/TerminalHUD/blob/dd1ee5d551352c69407ae349323d4ec179f2362e/tests/terminalhud-tests/windows-terminal/evidence/show8-phased.txt)
and [windowed control](https://github.com/TerminalHUD/TerminalHUD/blob/dd1ee5d551352c69407ae349323d4ec179f2362e/tests/terminalhud-tests/windows-terminal/evidence/windowed-show8.txt)
remain in Git history. Their diagnostic commands and event-monitor scaffolding
were removed after the Active-creation decision.

## Coverage boundary

Startup-focus observations do not establish fullscreen bounds, backend
Active/Passive/Hidden behavior, topmost, physical click routing, 35%/0% background
with opaque text, or session survival through hide/controller quit. The
[Active acceptance proof](windows-terminal-proof.md) measures those separately.
Installed Stable fragment-cache recognition, Preview and complete ordinary
profile cloning also require separate evidence. Portable same-version results
must not be presented as coverage of those cases.


## First normal Active acceptance run

The [first Active trace](../tests/terminalhud-tests/windows-terminal/evidence/active-run1.txt)
used reviewed cleanup revision `fd7c8aa81ad9825750ec6367f1bca7f4b30ff02c`,
fixture SHA256 `6a7c4331fc3094f49699b2103cf110943e38ae73a8118efe505305c01a2d12ef`
and synthetic settings SHA256
`1c2a145684f8d781bfed5124269dff957b80a0a9e8294c554f6eb62988cc10bf`.
Normal adapter Active creation verified owner PID 44696/HWND 13176914, fixture
PID 11884 and session `{b01d8c62-4999-49ee-9fb1-5bdaf3210c87}`. The controller
captured owned sink HWND 17239894 before launch, confirmed Managed Active, and
native checks confirmed foreground, primary fullscreen bounds and topmost.

The Active background sample was exactly RGB (26,104,143), matching 35% black
over the owned blue sink. The proof then stopped: it found zero fixture white
samples with every channel at least 245, below the required count ten. This is
unproven text rendering, not a demonstrated opacity defect. Passive, Hidden,
physical click and successful-quit assertion phases were not reached; no input
was injected. The error path attempted controller Quit without reporting a
restore failure, then closed the nonce fixture/target and destroyed the sink.
No owned processes remained; the preexisting Terminal witness and external
settings/fragment byte baselines stayed unchanged. Exact logs, original/normalized
synthetic settings, state, markers and fixture bytes were archived separately.

Read-only follow-up found a fixture inconsistency: its SGR 97 requests palette
bright-white, while the verified runtime's Campbell table defines that as
`#F2F2F2` (242), below the unchanged threshold. The synthetic profile overrides
default foreground to white but supplies no palette override. Microsoft's
[SGR contract](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#text-formatting)
distinguishes palette selection from explicit RGB. The handshake also preceded
output and flush, and the glyph block started at row 10 without recorded DPI.
These are fixture/readiness hypotheses, not measured causes of the zero count.
A separately reviewed correction will use explicit RGB white, early foreground
glyph blocks and a correlated output-flushed marker; acceptance thresholds and
background sample remain unchanged. Flush is not a GPU paint guarantee.


## Corrected fixture and foreground-restoration result

The [second Active trace](../tests/terminalhud-tests/windows-terminal/evidence/active-run2.txt)
used independently reviewed revision `8c5cbc2c0ebdb313aeb95d9ccb5e4f9dba92e920`,
fixture SHA256 `9a264438b3360ed7daee480f6b58a66dcbf636bf4167f3e669a395fc9bf0b79e`
and the unchanged original synthetic settings. Explicit RGB white foreground
blocks near the origin and exact output-flushed/session correlation corrected
fixture ambiguity without lowering the pixel predicate or expanding its region.

Normal Active creation verified owner PID 21532/HWND 18026326, fixture PID 36104
and session `{cd73e5d4-f1a5-42af-b0ee-e5bd0ef4168a}`. Native Active focus,
fullscreen/topmost and output-flushed correlation passed. On the 2560×1600
display, the unchanged 512×512/step8 scan found 1,562 near-white samples, including
actual RGB (255,255,255) at (168,8). Background RGB (26,104,143) exactly matched
the 35% expectation. This positively measures Active fixture text/background
composition; it does not establish every glyph or other renderer configuration.

The next RequestPassive returned a saved-focus warning `FocusRefused`, which the
strict proof wrapper treated as failure. Source maps that variant specifically
to `SetForegroundWindow(saved_hwnd)` returning zero, after saved PID/thread,
retained process lifetime and visible/enabled checks passed. A failed identity
check would report `InvalidTarget`; a later observed-foreground mismatch would
report `Timeout`. Exact foreground/class/PID/thread at that call were not logged,
so those numeric details cannot be reconstructed after cleanup. The controller's
warning path completes its own Passive observation, but the independent sink,
Passive renderer, click, Hidden/session-loop and successful-quit assertion phases
were not reached. No physical input occurred.

This transition was called programmatically after launch and pixel observation,
without a fresh hotkey/menu/foreground CLI action. It is not evidence that a
legitimate user-triggered request has the same foreground eligibility. The backend
reported the refusal and released owned target focus through its existing safe
path; no grant, synthetic input or focus bypass was attempted. Error cleanup
attempted controller Quit, closed only the nonce fixture/target and destroyed
the sink. No owned process remained; the existing-Terminal witness and external
configuration/fragment hashes stayed unchanged. Exact output marker, state,
settings, logs and binaries are preserved separately. Focus restoration and the
unreached acceptance phases remain unproven.


## Final-focus clarification and censored Passive capture

The [third Active trace](../tests/terminalhud-tests/windows-terminal/evidence/active-run3.txt)
used reviewed revision `0bf873412734d743aeada41c7139076f094d9760`, fixture SHA256
`8ea51fb059194233ba180f9ac980d389f34f2cb4b480307f82a48e3774707da9` and unchanged
synthetic settings. It confirmed Active bounds/state and the same strict RGB
result (1,562 white samples). Owner PID 46608/HWND 12324786 and fixture PID 35264
remained correlated with session `{dffa4a16-4f12-46f7-b10e-e040916a83aa}`.

RequestPassive returned confirmed Passive with the direct API warning retained.
The new final observation established exact saved sink HWND 31002472/PID
47080/thread 23252/class identity as foreground, and all unchanged strict native
Passive state/style/focus assertions passed. The warning therefore did not
represent an unmet final saved-focus requirement in this run. This required no
input, permission grant, shortcut or backend behavior change.

Passive pixel capture then exceeded the outer runner's 45-second deadline. The
runner requested only its nonce fixture's exit; the ongoing sample subsequently
reported changed target/session identity because that cancellation closed it.
That identity message is a consequence of cancellation, not independent evidence
of stale ownership. Passive rendering remains censored/inconclusive, not a
renderer failure. Physical click, Hidden/session-loop and successful-quit assertion
phases were unreached; no input occurred. Cleanup left zero owned processes and
unchanged external Terminal/configuration witnesses.

The old 4,096-point path repeated live identity/hit tests and screen `GetPixel`
per point; its retry deadline was checked only after a whole scan. A bounded
owned-region in-memory capture will replace those repeated readbacks while
preserving exact coordinates, step, color thresholds, renderer settings and the
45-second outer budget. All run3 artifacts, output marker, normalized settings,
state, original fixture and manifests were archived before this change.
