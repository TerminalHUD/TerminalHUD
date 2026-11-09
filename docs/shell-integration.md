# Shell integration

Prompt-aware Escape is implemented by the shell's line editor, not by a global
Escape hook.

## Rationale

Windows Terminal receives keyboard input even while Neovim or another terminal
program is running. An external keyboard hook cannot determine whether the
terminal application consumed Escape. The shell line editor, however, is active
only while the shell is accepting prompt input.

Therefore:

- At a supported shell prompt, the line editor can request Passive.
- While Neovim, `less`, `fzf`, SSH, or another interactive program runs, that
  program receives Escape normally.

## IPC command

Integrations conceptually invoke:

```text
terminalhud passive
```

This is a lightweight client request to the running per-user TerminalHUD
instance. It should return quickly and silently when successful.

## Deactivation rule

The default integration requests Passive only when:

- the HUD is Active;
- the shell line editor is accepting input;
- the current command line is empty; and
- no completion, history, or similar interactive menu needs Escape.

If these conditions are not met, the shell's ordinary Escape behavior is
preserved where the shell API allows it.

## Initial integrations

### PowerShell

- Support PowerShell with PSReadLine.
- Account for both Emacs and Vi editing modes.
- Preserve existing user key handlers where practical.
- Do not overwrite a PowerShell profile without explicit approval and backup.

### WSL Bash

- Use Bash/Readline integration inside the selected WSL distributions.
- Invoke the Windows client through WSL interoperability.
- Preserve Alt/Meta escape-prefix behavior.
- Handle multiline prompts and completion state conservatively.

### Git Bash

- Use Bash/Readline integration appropriate to Git for Windows.
- Resolve the installed `terminalhud.exe` through its stable per-user install
  location.
- Preserve existing Readline configuration.

## Installation UX

CLI commands list discovered shells and expose explicit preview, install, and
remove operations for each integration. Before writing, the command shows:

- the file to be changed;
- the exact snippet or sourced integration file;
- whether an existing binding conflicts; and
- where the backup will be written.

Prefer adding a small, clearly marked source/include line over embedding a large
script in a user's startup file. Installation and removal must be idempotent.

The tray menu may open documentation or invoke the same commands, but it does
not need a dedicated configuration window.

## Fallback

Prompt-aware Escape is a convenience, not the only exit path. `Win+Backtick`
always toggles Active to Passive even when a shell has no integration or an
interactive program does not use Escape.


## Implemented MVP scripts

The package includes explicit Preview, Install and Remove scripts. See the
[shell integration guide](../integrations/shells/README.md) for supported modes,
backup behavior, versioned upgrade resolution and isolated tests. The MVP supports
PSReadLine Windows mode and Bash Readline Emacs mode. PSReadLine Emacs/Vi and
custom Escape handlers are detected and preserved with a warning.
