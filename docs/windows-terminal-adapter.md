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
| `opacity` | Integral percentages in 0–100 (including integral real notation such as `35.0`); fractional percentages are invalid |
| `backgroundImageOpacity` | Numbers in 0–1 |
| `backgroundImage`, `colorScheme` | Raw strings; resources, scheme definitions and color contents are not resolved |
| `unfocusedAppearance` | Object containing the five appearance fields above |

An unfocused object wins by owning layer; missing nested fields inherit that
owner's focused appearance chain. Lower-priority unfocused objects are not
recursively merged. A direct `unfocusedAppearance: {}` therefore inherits its
own focused opacity rather than a fragment's unfocused zero. A defaults-owned
unfocused object inherits defaults-focused fields; a fragment-owned object
inherits fragment-focused fields, still marked as candidates because an
unsupplied inbox unfocused object could win first. Legacy `acrylicOpacity` also affects opacity in the pin. An alias without a
same-layer modern opacity makes that layer's focused/unfocused opacity explicitly
unsupported, rather than falling through to a lower-priority modern value. A
higher-priority modern value, or a same-layer modern value beside a numeric alias,
can win; legacy conversion itself is not modeled. Nonnumeric aliases are refused.
See [the pinned appearance parser](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/AppearanceConfig.cpp#L124-L125)
and [percentage converter](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalSettingsModel/TerminalSettingsSerializationHelpers.h).

The owning-layer inheritance follows pinned
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

## Registered package discovery

On Windows, `discover_registered_terminals()` queries both current-user package
families and returns `RegisteredTerminalDiscovery`. Stable uses
`Microsoft.WindowsTerminal_8wekyb3d8bbwe`; Preview uses
`Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe`. These family names and expected
settings locations are documented by [Microsoft's Terminal installation guide](https://learn.microsoft.com/en-us/windows/terminal/install#settings-json-file).
The [pinned package manifest](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/CascadiaPackage/Package.appxmanifest)
identifies the Stable package and the `WindowsTerminal.exe` component. The
executable path here is a layout expectation, not a manifest or file check.

Each channel independently reports an API failure or all returned registrations;
an empty successful list means that family had no registrations at query time.
No version wins automatically. Each candidate retains the queried full/family
name, channel, parsed identity (name, publisher ID, resource ID, package version,
architecture), registered installation path and expected application paths.
Identity, installation and Local App Data failures remain separate, so an
unavailable installation path does not erase independently available identity or
settings-location metadata. Package version is distinct from executable file
version and running Terminal version; neither of the latter is inspected.

Discovery uses documented direct APIs:

- [GetPackagesByPackageFamily](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-getpackagesbypackagefamily)
  enumerates registrations for the current user.
- [PackageIdFromFullName](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-packageidfromfullname)
  with `PACKAGE_INFORMATION_BASIC` parses identity into aligned owned storage.
  It does not load the package or establish a running process's identity.
- [GetPackagePathByFullName](https://learn.microsoft.com/en-us/windows/win32/api/appmodel/nf-appmodel-getpackagepathbyfullname)
  supplies the registered installation root.
- [SHGetKnownFolderPath](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath)
  resolves `FOLDERID_LocalAppData` with the current-user token and
  `KF_FLAG_DONT_VERIFY | KF_FLAG_NO_PACKAGE_REDIRECTION`. These
  [flags](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ne-shlobj_core-known_folder_flag)
  avoid existence checks and packaged-caller private redirection. Neither
  `CREATE` nor `INIT` is used. Task memory is freed on success and failure.

For an empty-resource x86, x64 or ARM64 package with the expected package name and
publisher ID, expected paths are `<registered root>/WindowsTerminal.exe` and
`<LocalAppData>/Packages/<family>/LocalState/settings.json`. Resources, bundle
identities (including resource ID `~`), neutral and unknown architectures retain
identity metadata but receive `UnsupportedApplicationPackage` for application
paths. Other raw architecture values are preserved. A name/publisher mismatch is
a typed identity error. This subset does not claim application-package type,
manifest validity, file existence, signature, hash or binary architecture checks.

Each variable-size API query allows at most four calls, including initial sizing.
Limits are 128 package names, 65,536 UTF-16 units for the family-name buffer,
65,536 bytes for an identity buffer and 32,768 UTF-16 units for paths. Required
sizes are capped before allocation. Returned sizes/counts are checked against
owned storage; pointers are converted to bounded aligned offsets before string
reads. Strings must terminate inside the reported prefix and decode as UTF-16;
duplicate names, malformed replies, exhausted retries and native errors remain
explicit failures. Identity storage is `u64`-aligned before the API call; no
borrowed native pointer escapes. The known-folder API instead owns its allocation
and guarantees a terminated string; its owner is retained through a capped scan
and freed exactly once.

This is a non-atomic observation of two registered package families, not a complete
installation inventory or a stable snapshot. Packages can disappear or change
between queries. It reads no settings, manifests, fragments or executable bytes,
uses no PATH lookup or command execution, and does not inventory unpackaged
installs, package extensions, providers, policy, host processes or cached catalogs.
It changes no files, registrations, processes, windows or existing launch gate.
It neither authenticates an executable nor establishes runtime recognition,
installed readiness, safe fallback or permission to launch.

Portable injected-response tests exercise buffers and partial metadata failures.
Windows-only tests enumerate read-only without requiring Terminal installed and
parse a synthetic package identity using the documented basic identity API. They
launch no Terminal or shell. A native isolated feature check is
`cargo check -p terminalhud-windows-terminal --lib --locked`; adapter Windows API
features are declared directly, independently of app feature unification.
Actual user's-host probing requires separate exact-source/runner review; no such
host probe is performed by this change.

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

## Installed settings and launch preparation

[ADR 0010](decisions/0010-installed-terminal-window.md) supersedes the earlier
separate-process requirement. `Catalog::from_sources` consumes current user
JSONC, optional actual inbox defaults, ordered fragments and generated provider
results. `prepare` selects an explicit/default source GUID and produces an owned
profile value, fragment preview and original command/directory/environment.
`fragment_compatible` identifies native-default conflicts; such a conflict can be
represented by the proposed user-list entry and is not an unsupported user theme.
`validate_fallback` checks defaults/default-profile elevation and connection type
before an explicit bridge command can reach an unknown-profile fallback.

The source ordering is inline user fields, user defaults, fragment updates, then
provider/inbox fields. Font aliases normalize within each layer; font subfields
and light/dark schemes inherit independently, while axes/features/environment
maps remain atomic. The first unfocused object (including a present null) owns its
focused fallback. The adapter materializes that fallback so cloning does not
accidentally inherit a different profile's colors. Unknown properties are copied
as opaque values. The normal global configuration is neither exported nor rebuilt.

Native `read_inbox_defaults` reads resource 101 from the candidate-adjacent
`Microsoft.Terminal.Settings.Model.dll` using data/image-resource flags, without
executing its settings loader. `discover_generated_profiles` reads legacy WSL
registration and supports an unambiguous single traditional PowerShell install;
it does not start either shell. Modern WSL profiles come from their fragments.
Competing PowerShell layouts, unavailable provider data and other generators remain
unresolved until supported; source names are not permission to guess a command.
`read_profile_fragment` retains known media origin by resolving fragment-relative
icons, backgrounds, shaders and sounds before materialization, and generates
missing fragment GUIDs with Terminal's namespace/UTF-16 UUIDv5 rules.

`plan_wsl_launch` models Terminal's command-sensitive directory promotion for a
system `wsl.exe`, keeping original arguments and existing `--cd`/bare `~` behavior.
The bridge must use this plan when replacing the Terminal launch command; simply
inheriting the bridge cwd does not preserve a WSL Linux directory. Shell launch
and profile/session authorization are application composition responsibilities.

The pure owned-entry editor and proposed setup policy are documented in
[ADR 0011](decisions/0011-owned-profile-entries.md). No real settings write is
performed by this adapter. Application code owns file receipts/backup/transactions,
refreshes input on startup/reload/next launch, and never closes live sessions for
settings updates. Successful disk preparation is not proof that an existing host
has reloaded a profile. On GUID mismatch the bridge refuses to execute the shell;
Terminal may already have opened an unmanaged fallback window. Report that case
without killing a shared process or silently selecting another shell.

An explicit bridge command changes Terminal's default environment policy to
inherit, even when the selected profile normally reloads environment variables.
`PreparedProfile::reload_environment` resolves
`compatibility.reloadEnvironmentVariables` (built-in default `true`), and the
installed launcher uses `start_with_command_and_environment` to emit exactly one
`--reloadEnvironment` or `--inheritEnvironment` option before the bridge command.
Existing launch entrypoints retain their previous argument behavior. See the
[Microsoft command-line documentation](https://learn.microsoft.com/en-us/windows/terminal/command-line-arguments)
and [pinned parser](https://github.com/microsoft/terminal/blob/7ff11ee107c984e236960ec948b450ccbc10187b/src/cascadia/TerminalApp/AppCommandlineArgs.cpp).
