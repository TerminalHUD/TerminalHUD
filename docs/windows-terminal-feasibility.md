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
