# ADR 0011: Proposed receipt-owned profiles at user-settings precedence

- Status: Proposed; explicit setup consent required before activation
- Date: 2026-11-09
- Would amend: ADR 0004's fragment-only storage policy

## Problem and native alternatives

Windows Terminal gives inline user profile fields precedence over user defaults,
then fragment updates and provider/inbox data. Its loader adds user defaults as
a most-important parent of every fragment profile. A new fragment therefore
cannot override a default opacity or reproduce a source profile whose inline font
size differs from the default. The supported launch arguments do not accept
arbitrary profile properties. Neither another runtime nor changing ordinary
profiles satisfies the installed-settings requirement.

Sources: [Terminal inheritance](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp),
[profile inheritance](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/Profile.cpp),
and [supported launch arguments](https://learn.microsoft.com/en-us/windows/terminal/command-line-arguments).

## Proposed change

After a local preview and explicit setup consent, add an independently identified
HUD profile to the existing user `profiles.list` (or legacy `profiles` array).
Keep a stable owned HUD GUID distinct from the source. The entry materializes
source inheritance, retains arbitrary opaque settings, and changes only identity,
HUD opacity and necessary Passive background behavior. The installed Terminal
continues to resolve the original global settings and named color schemes.

The adapter prepares an exact insertion block containing a marker, the new JSON
object and a trailing comma. The original document is not reserialized. Every
ordinary byte, comment, BOM and newline outside that block is preserved. A receipt
stores the exact GUID and block. Refresh/removal requires one actual AST profile
entry matching that unchanged block and GUID; a comment containing the block is
not ownership. Duplicate keys/GUIDs, collision, missing or edited owned material
stop the operation. No ordinary profile is renamed, removed or rewritten.

The app owns preview/apply, persistent consent, backup, receipt, concurrent-write
protection and rollback. Compare the original bytes under the transaction before
writing; a stale preview must fail. After consent, startup/reload may update only
the unchanged owned entry. Keep live sessions open and report next-launch/reload
requirements without claiming immediate renderer updates.

## Validation and limits

The adapter editor is pure and is tested only with synthetic in-memory documents.
The transaction uses temporary fixtures. This ADR does not approve changing the
real user's settings, installing a build, or launching visible desktop tests.
The root must present the concrete proposal and obtain approval before activation.

Initial editor input requires an existing explicit profile array. Missing profile
containers require an explicit setup path rather than inventing surrounding user
structure. Known native appearance/font aliases and nested inheritance are
modeled; unknown properties remain opaque. Actual provider commands must come
from supported installed discovery or explicit profiles, never guessed shells.
