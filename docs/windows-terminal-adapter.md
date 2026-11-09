# Windows Terminal adapter primitives

The adapter separates read-only `profiles` parsing/preview, supplied-byte
`assessment`, pure `launch`
identity/correlation, and Windows-only `native` process/window observations.
It has no runtime dependency on the Win32 backend or app. The intended app
composition converts a verified delegated launch into the backend's receipt;
title, class, and launcher PID alone never authorize acquisition.

`Catalog::parse` consumes supplied JSONC. It merges valid explicit profile defaults,
but does not resolve generated profiles, installed fragments, or dynamic sources.
A dynamic source without its own command line remains unresolved even when the
global defaults contain a command line. Invalid default identity/command keys are
stripped to match Terminal; no shell command is evaluated by parsing.
`Catalog::preview` rejects unresolved/elevated sources and GUID collisions. It
creates a separate profile, strips source/update identity, explicitly disables
elevation, and overrides focused/unfocused HUD appearance. Original settings and
profiles remain untouched. It is a preview, not a proof that Terminal loaded it.

`LaunchPlan` requests a fresh named fullscreen window at the supplied primary
monitor origin. Normal `native::start` and `start_with_command` return
`Unsupported` for Passive before spawning: the tested fullscreen startup path
activated transiently. Active callers must still establish safe runtime-profile
and fallback preconditions; this does not certify rendering compatibility.
The returned process can delegate to another Terminal process. Native correlation
retains live identities and checks every ancestry edge up to the exact candidate
owner PID/executable, without traversing irrelevant upstream parents. It
excludes preexisting HWNDs, and checks the exact installed executable in addition
to a nonce/profile/session shell handshake. It does not make intercall HWND
lifetime changes atomic. The [controlled portable proof](windows-terminal-proof.md)
uses isolated synthetic settings; `preview` only writes a fresh task directory.

The [native feasibility report](windows-terminal-feasibility.md) records the
startup findings behind the Active-creation decision and the limits of that evidence.

## Installed-version source expectations

These observations use Microsoft Terminal v1.24.12741.0, commit
`7ff11ee107c984e236960ec948b450ccbc10187b`. They are static source expectations,
not native GUI/input/rendering evidence:

- [GetProfileForArgs](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettings.cpp#L735-L780)
  can fall back from an unknown requested profile. With an explicit command line
  it may select a command-matched profile or `ProfileDefaults`; otherwise it
  selects the ordinary default. Supplying a fixture command alone cannot prove
  non-elevated ConPTY launch or absence of inherited launch behavior. Do not
  launch against an unverified fragment cache. No installed runtime-catalog guard or
  complete profile-source resolution is implemented yet.
- [AppLogic](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalApp/AppLogic.cpp#L304-L340)
  watches the main settings filename. A fragment on disk is insufficient reload
  evidence; do not write main settings or restart unrelated sessions to force it.

Native proof must separately establish fullscreen bounds, Active creation and focus restoration,
Active/Passive/Hidden observations, actual click routing, background transparency
with opaque text, session survival, and exact owned-file/session cleanup. The corrected isolated portable component proof passed background transparency,
opaque-text sampling and click routing as recorded in the
[feasibility report](windows-terminal-feasibility.md). That evidence covers its
synthetic profile and owned sink; application composition evidence and installed
runtime/profile recognition require their own validation.
Stable/Preview and reload coverage must be recorded individually.

## Supplied profile assessment

`assess_snapshot(SuppliedSnapshot, requested_guid, fallback_commandline)` consumes
main settings JSONC plus explicit `SuppliedFragment` documents in the
`TerminalHUD` namespace. Document identifiers are caller labels, never paths to
open. It performs no discovery, writes, command evaluation, native calls or
spawning, and is not connected to `Catalog::preview`, `LaunchPlan` or native
launch gating. No input asserts complete discovery, host freshness or permission
to launch.

`ProfileAssessment` reports the requested identity only, per-property raw values
and provenance, possible fallback branches and typed findings. A missing requested
GUID never selects the normal default. With a nonempty fallback command the only
reported branch set is unresolved command-prefix matching or `ProfileDefaults`;
without one, the ordinary default is merely possible. Matching, normalization and
fallback safety are not implemented, even for an apparent command-prefix match.

The bounded supplied inheritance is direct user profile, user profile defaults,
then one independent fragment contribution. **The real loader places inbox
profile defaults before fragments.** Inbox bytes are not supplied here, so every
fragment-derived property is a `Candidate` with `InboxDefaultsUnresolved`, raw
value and exact `(namespace, document_id)` provenance. It is never a resolved
`Supplied` expectation. Wholly absent values remain `MissingBaseLayer`; built-in
or runtime defaults are never invented. Direct/default supported values can be
`Supplied`, but all values remain raw settings expressions rather than evaluated
commands, expanded paths/environment or resolved color resources.

Supported properties are deliberately limited:

| Properties | Supported supplied shape |
| --- | --- |
| `commandline`, `startingDirectory` | Strings; empty commands remain unsupported/unresolved |
| `environment` | One atomic object of string values; no per-variable merge or expansion |
| `elevate`, `hidden`, `useAcrylic` | Booleans |
| `connectionType` | GUID string; nonzero values produce a refusal |
| `opacity`, `backgroundImageOpacity` | Numbers in 0–100 and 0–1 respectively |
| `backgroundImage`, `colorScheme` | Raw strings; resources, scheme definitions and color contents are not resolved |
| `unfocusedAppearance` | Object containing the five appearance fields above |

An unfocused object wins by owning layer; missing nested fields inherit that
owner's focused appearance chain. Lower-priority unfocused objects are not
recursively merged. A direct `unfocusedAppearance: {}` therefore inherits its
own focused opacity rather than a fragment's unfocused zero. A defaults-owned
unfocused object inherits defaults-focused fields; a fragment-owned object
inherits fragment-focused fields, still marked as candidates because an
unsupplied inbox unfocused object could win first. This follows pinned
[Profile.cpp `LayerJson` and `_FinalizeInheritance`](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/Profile.cpp#L211-L265),
[profile parent precedence](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L1063-L1106)
and [inbox/user defaults finalization](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/CascadiaSettingsSerialization.cpp#L485-L499).
The source pin is v1.24.12741.0 / `7ff11ee107c984e236960ec948b450ccbc10187b`,
not a support range or claim of future source stability.

Every profile needs a nonzero explicit GUID. One same-GUID user entry may overlay
one fragment; duplicate user GUIDs, duplicate fragment identities and competing
fragment GUID contributions are refused. Multiple different documents in the
same namespace are supported. Duplicate JSON keys, including escaped equivalents
and nested keys, are rejected before conversion to lossy JSON maps. Only JSON
comments and trailing commas are enabled; JSON5 syntax and missing commas are
rejected. Malformed
supported properties and containers have separate findings from unsupported
forms. Null property values, null unfocused objects and object-shaped color
schemes remain unsupported/unresolved rather than taking a guessed inheritance
path. The loader's invalid defaults identity/command keys are cleared before
profile validation, matching the source pin.

`updates`, generated GUIDs, other fragment namespaces and fragment source keys
are unsupported. A nonempty user `source` is unresolved unless it identifies an
enabled supplied TerminalHUD fragment with that GUID. Disabled TerminalHUD
fragments do not contribute properties; unresolved providers anywhere in the
supplied list produce findings. Other settings, schemes, font settings, actions,
providers, machine/package fragments, policy and inbox profiles are not modeled.
This is not a complete Windows Terminal settings interpreter.

Every result, including malformed input, retains unproved fallback safety,
inventory completeness, input freshness, runtime recognition, loaded catalog,
dedicated process and launch environment findings. Changed supplied bytes can
change candidates but never prove a running host loaded them. Existing/cached
hosts and namespace completeness require separate native inventory and runtime
evidence; the API has no host assertion or launch-permit status. Elevated or
nonzero-connection supplied values (including possible fragment candidates)
produce additional typed refusals. This API establishes no installed readiness.

## Focused checks

After the workspace's locked dependency fetch, run:

```sh
cargo test -p terminalhud-windows-terminal
cargo clippy -p terminalhud-windows-terminal --all-targets -- -D warnings
cargo fmt --all -- --check
python3 scripts/check_repository.py
git diff --check
```

The same Cargo test/lint commands run on native Windows. Focused tests cover
profile preservation/override, rejected launch sources, dynamic/default command
ambiguity, isolated arguments, launch correlation, and malformed identities.
They do not launch Terminal. The optional `terminal-proof preview <absolute-exe>`
example creates a synthetic fragment preview and record in a fresh task directory;
it never installs a fragment or edits main settings. Its `--shell` fixture checks
nonce, `WT_PROFILE_ID`, and `WT_SESSION`, then waits for a nonce-scoped cleanup
request. Native tests additionally check owner-prefix and CLI-command refusal.
