# HudSwitch state machine

HudSwitch models interaction state independently of TerminalHUD. The initial
product exposes one controller and one managed target, but the library must not
encode that product limitation into global static state.

## States

### Active

- Target is visible and topmost.
- Target can receive mouse and keyboard input.
- Target owns foreground focus.

### Passive

- Target is visible and topmost.
- Pointer input passes through to windows beneath it.
- Keyboard focus belongs to another window.

### Hidden

- Target is hidden without being closed or minimized.
- Hidden stores `returnState`, which is either Active or Passive.
- Restoring visibility returns to `returnState`.

Target availability is tracked separately from the three states. A target may
be absent because it has not been launched or because the user closed it.

## Transitions

| Trigger | From | To | Notes |
| --- | --- | --- | --- |
| Interaction toggle | Passive | Active | Capture current foreground window, then focus target |
| Interaction toggle | Active | Passive | Restore captured foreground window |
| Interaction toggle | Hidden | Active | Show and focus; remembered state is discarded |
| Interaction toggle | Absent | Active | Launch, acquire window, and activate |
| Visibility toggle | Passive | Hidden(Passive) | Hide and remember Passive |
| Visibility toggle | Active | Hidden(Active) | Hide, remember Active, and restore prior focus |
| Visibility toggle | Hidden(Passive) | Passive | Show without taking focus |
| Visibility toggle | Hidden(Active) | Active | Show and take focus |
| Visibility toggle | Absent | Passive | Launch and make click-through without stealing focus |
| Prompt-aware Escape | Active | Passive | Shell integration requests the transition |
| Unexpected focus loss | Active | Passive | Prevent a transparent fullscreen input blocker |
| Managed window closed | Any | Absent | TerminalHUD remains running |
| TerminalHUD quit | Any present state | Unmanaged | Restore original styles and leave target open |

## Invariants

- Passive is visible, unfocused, and click-through.
- Active is visible, focused, and not click-through.
- Hidden is never focused and never blocks input.
- A transition restores only the style bits HudSwitch changed.
- HudSwitch records original window styles before its first mutation.
- HudSwitch does not change application-specific opacity or rendering.
- A failed activation must not leave the target visible but unexpectedly
  input-blocking.
- State transitions are serialized; overlapping hotkey, focus, lifecycle, and
  IPC events cannot partially apply two states.

## Focus restoration

When entering Active from Passive, HudSwitch records the current foreground
window. When leaving Active, it attempts to restore that window if it is still
valid and eligible for activation. If restoration fails, HudSwitch leaves focus
with Windows rather than repeatedly forcing it.

When restoring Hidden(Active), the application that is foreground immediately
before restoration becomes the new focus-restoration target.

## Failure handling

Every transition should be transactional where practical:

1. Validate the target handle.
2. Record state needed for rollback.
3. Apply window styles and visibility.
4. Apply focus and Z-order behavior.
5. Confirm observable state.
6. Commit the logical state, or roll back to a safe Passive/Hidden condition.

