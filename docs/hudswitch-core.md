# Portable HudSwitch controller contract

`hudswitch-core` implements the state machine in [state-machine.md](state-machine.md).
It has no OS, terminal, UI, or external crate dependencies. Each `Controller<B>`
owns one backend and at most one managed target; separate controllers can coexist.
The one-target initial product policy is not global library state.

## State and event delivery

`State` is Active, Passive, or Hidden with an Active/Passive return state.
`Status` reports target availability separately: Absent, Managed, or Unmanaged
(after successful explicit shutdown). A managed target with `state: None` means
its presentation is unconfirmed, not a fourth user-facing window state.

Hosts deliver interaction/visibility toggles, generic passive requests, focus
loss, target closure, and quit to `dispatch(&mut self, Event)`. The exclusive
borrow covers the complete transition. A host with concurrent producers uses one
queue or a mutex around its controller. Backend callbacks enqueue notifications
rather than recursively dispatching while that lock is held. The core has no
async runtime or global event loop.

Lifecycle notifications carry the controller's opaque `TargetId` from `status()`.
The ID changes on acquisition, so a delayed old-window closure/focus notification
cannot affect a replacement target. A stale target discovered during validation
becomes Absent; a toggle then launches its replacement with the requested intent.

`Controller::new` preserves that requested-state creation behavior.
`Controller::with_creation_state(backend, state)` selects a visible state for
every transition that creates a missing target. The policy is applied after
target validation and before capabilities, focus capture, and acquisition, so
stale-target replacement follows the same policy atomically. Existing targets
and events that do not create targets are unchanged. TerminalHUD selects Active;
the library retains generic Passive creation support.

A passive request only changes Active to Passive. The caller decides when such
a request is appropriate; the core does not know about prompts, shells, or keys.
A FocusLost event preserves the foreground window the user selected. Hiding or
restoring a Passive target also preserves the current foreground. Leaving Active
normally attempts the previously captured foreground once. A failed attempt is
returned as `Outcome::focus_restore_error`; the backend then relinquishes target
focus if it still owns it and lets the OS choose. It must never repeatedly force
an unavailable saved foreground.

## Backend boundary for platform implementations

The `Backend` trait uses associated Target, Focus, Original, and Error types.
Their concrete representations belong to the backend; platform handles do not
enter portable public data structures. Target/snapshot tokens must not close or
terminate the target application when dropped.

| Method | Required behavior |
| --- | --- |
| `capabilities` | Advertise complete launch/Active/Passive/Hidden support for this platform/session. Unsupported requests are rejected before acquisition, capture, or mutation. Active also requires at least one safe fallback presentation. |
| `acquire(intent)` | Launch/acquire only the intended target and return its original owned-state snapshot before any controller mutation. Passive creation must never steal focus, including briefly during launch. On error, the backend owns cleanup of partial acquisition and must leave any created target ordinary and usable. |
| `is_valid` | Confirm the target's identity and lifetime; distinguish closed (`false`) from an inability to validate (`Err`). |
| `foreground` | Return a token for the current external foreground. The controller captures this before an Active launch and each subsequent activation, including restoring Hidden(Active). |
| `present` | Apply visibility, topmost, and pointer/input style intent; preserve all unrelated style bits. Leave appearance/opacity to the application/adapter. Do not activate unrelated windows. |
| `activate` | Focus the target or return an error. The controller confirms observed focus before committing Active. |
| `restore_focus` | Attempt a saved foreground only if it is still valid and eligible. Do not retry or substitute arbitrary windows. |
| `release_focus_if_owned` | Release target focus only while it still owns it; preserve another foreground selected meanwhile. |
| `observe` | Report visible, topmost, click-through, and focused facts. Success must describe actual observable state, not merely the last request. |
| `restore_original` | Restore only owned changes from the snapshot, including visibility as appropriate, and leave the target open and usable. Success certifies restoration is complete. |

Every operation touching a target must revalidate identity as needed: a window
can close between any two calls. The generic backend tests will validate Win32
semantics with a purpose-built window before terminal integration.

## Commit, rollback, and shutdown

The controller checks capabilities before starting a new requested transition.
It marks presentation unconfirmed when mutation begins, applies presentation and
focus, then observes the resulting invariant. Only a confirmed observation commits
the requested logical state.

On mutation/activation/observation failure, it attempts supported safe Passive,
then Hidden(Passive). Each fallback is also observed. The returned
`TransitionError` retains the initiating stage/cause, recovery outcome, and
fallback failures. If the target closed, recovery reports Absent. If no safe
presentation can be confirmed, the target and original snapshot remain owned,
availability stays Managed, and `state` is None. Ordinary transitions then return
RecoveryRequired rather than guessing a state.

`recover()` explicitly retries safe presentation while preserving the current
external foreground. `Quit` can also be retried directly. A failed original-state
restoration retains the same snapshot and target ownership, and attempts safe
presentation; it never falsely commits Unmanaged. Successful Quit drops management
after restoration and does not close the target. It is idempotent; a stopped
controller rejects further ordinary commands.

The host must explicitly dispatch Quit and handle any error before discarding its
controller. Dropping the controller is not a substitute for confirmed shutdown.
Crash recovery and process-lifecycle supervision belong to backend/application
work; portable tests do not establish those platform guarantees.

## Tests and current limits

`cargo test -p hudswitch-core --locked` runs the registered black-box suite from
`tests/hudswitch-core-tests/`. Its fake backend records calls, models visible/input/
focus state and owned versus unrelated styles, and injects failures after partial
mutation. Tests cover transitions, focus preservation, launch ordering, generation
filtering, unsupported capabilities, safe/unconfirmed recovery, repeated quit,
and serialized host events.

These are portable contract tests. They do not establish actual Win32 focus,
click-through, window acquisition, or terminal behavior. Those require the
backend test window and native desktop evidence in subsequent tasks.
