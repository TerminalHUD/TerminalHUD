# ADR 0010: Use the installed Terminal with a dedicated HUD window

- Status: Accepted
- Date: 2026-11-09
- Supersedes: the dedicated-process clause in ADR 0008

## Decision

Use the user's installed Windows Terminal. Launch a uniquely named HUD window,
allowing Terminal to share its owner process with ordinary windows. Do not ship
or install another Terminal runtime. Preserve the selected normal profile and
all installed global settings; apply only agreed HUD presentation/input changes.

The launch bridge checks the expected profile GUID and session identity before
executing the selected shell. Window acquisition additionally checks the unique
launch identity, new HWND and retained owner-process epoch. PID equality alone
is neither window ownership nor authority to terminate a shared process.

## Consequences

A saved profile does not prove a running Terminal host has reloaded it. An unknown
profile can open an unmanaged fallback window before the bridge rejects its GUID.
Validate fallback connection/elevation before launch, report recognition failure,
and let the user reload/reopen Terminal when needed. Never force-restart Terminal
or close unrelated sessions. Existing sessions remain open during settings refresh.

The source parser and installed package discovery provide input to launch
preparation; the older seven-finding diagnostic assessment is not a launch gate.
The profile-storage policy is separately addressed by the proposed ADR 0011.
