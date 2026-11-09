# Prompt-aware Escape

At an empty primary prompt, Escape requests Passive from the existing same-user
TerminalHUD owner. It never starts TerminalHUD or a terminal. The request runs
outside the line editor without waiting for IPC; a missing installation or owner
is a silent no-op. `Win+Backtick` remains the unconditional fallback.

Supported modes:

| Shell | Mode | Behavior |
| --- | --- | --- |
| Windows PowerShell 5.1 / PowerShell 7 with PSReadLine | Windows (default on Windows) | Empty buffer requests Passive; nonempty buffer keeps RevertLine/Abort |
| WSL Bash / Git Bash | Bash 5+, Readline Emacs, promptvars enabled, positive keyseq-timeout | Empty primary prompt requests Passive; arrows and Alt/Meta prefixes remain bound |
| PSReadLine Emacs or Vi; Bash Vi; custom standalone Escape/prefix handlers | Unsupported | Warning on load; existing bindings remain unchanged |

Handler preservation uses a small read-only adapter to PSReadLine's private
2.0–2.4 dispatch layout, because its public key-handler metadata exposes display
labels rather than identity. It accepts only the validated ConsoleKeyInfo or
PSKeyInfo dictionary layouts, exact single-invocation static built-in delegates, and the original
installed handler object on unload. Custom labels such as `RevertLine` or
`TerminalHUDPassive` do not grant ownership. Other versions, unavailable private
metadata or changed layouts produce a warning and leave bindings unchanged.
The integration writes bindings only through public PSReadLine commands.

PSReadLine owns Escape during its completion/search menus; the hook runs only
when normal line editing dispatches Escape. Bash incremental search also consumes
Escape itself. Bash completion that leaves input in the buffer keeps Escape in
the editor. A small zero-output prefix in PS1 and PS2 distinguishes continuation
prompts, where the editable line can be empty. If a prompt framework replaces
those prefixes, the hook stops requesting Passive. Source the integration after
prompt customization. Do not disable Readline's finite key sequence timeout:
standalone Escape waits for that existing timeout so longer sequences still work.
Undefined longer Escape keys retain Readline abort behavior. The integration
records the existing prefix tree and fills its undefined entries with abort
before adding the shorter handler, preventing unknown Alt/CSI keys from falling
back to Passive. Custom ambiguous shorter-prefix fallbacks are declined.

Neovim, `less`, `fzf`, SSH and other foreground programs receive their ordinary
terminal input: these hooks run inside the shell line editor, without a global
keyboard hook. Bash uses Windows PowerShell interoperability for the Windows
client; WSL interoperability must be enabled. Windows execution policy is
respected. No policy is changed by these scripts.

## Preview, install and remove

The Windows package includes `integrations/shells/manage.ps1`. Run it explicitly
in PowerShell, using its actual extracted package path. The examples below assume
you have changed to that directory. Preview does not create a profile or backup.
Install and Remove are the explicit write operations; both show their plan and
back up an existing file before changing it.

For the current PowerShell host's current-user profile:

```powershell
.\manage.ps1 -Operation Preview -Shell PowerShell -ProfilePath $PROFILE
.\manage.ps1 -Operation Install -Shell PowerShell -ProfilePath $PROFILE
.\manage.ps1 -Operation Preview -PreviewAction Remove -Shell PowerShell -ProfilePath $PROFILE
.\manage.ps1 -Operation Remove -Shell PowerShell -ProfilePath $PROFILE
```

Use the same shell, installation root and profile path for removal. Repeat in
the other PowerShell host if you want both inbox PowerShell and PowerShell 7.
Load PSReadLine before the added block. Restart the shell after installing; after
removal restart it or run `Remove-Module TerminalHUD` in PowerShell to restore
the old handler. Already-running Bash shells keep their hook until they exit.

For WSL Bash, explicitly name the selected distribution's `.bashrc` and the
POSIX path to the Windows installation. Replace the example user/distribution
with your actual paths; no distribution is discovered or modified automatically:

```powershell
.\manage.ps1 -Operation Preview -Shell WSLBash `
  -ProfilePath '\\wsl.localhost\Ubuntu\home\operator\.bashrc' `
  -BashInstallRoot '/mnt/c/Users/operator/AppData/Local/TerminalHUD/install'
```

For Git Bash, explicitly select its `.bashrc` and shell-visible install root:

```powershell
.\manage.ps1 -Operation Preview -Shell GitBash `
  -ProfilePath 'C:\Users\operator\.bashrc' `
  -BashInstallRoot '/c/Users/operator/AppData/Local/TerminalHUD/install'
```

Use `-Operation Install` to apply the preview, and `-Operation Remove` to remove
the same block. If a Git Bash login profile does not already source `.bashrc`,
select the startup file it actually reads instead. No other startup files are
rewritten. `-InstallRoot` can name a different receipt root when generating a
snippet; the production IPC client always uses the current Windows user's
standard `%LOCALAPPDATA%\TerminalHUD\install` location.

Preview shows the target, exact owned block, proposed backup and binding-check
status. Effective shell bindings cannot be inspected from another shell without
running its configuration, so conflicts are checked at integration load before
binding. Custom Escape handlers, PSReadLine Escape chords and unsupported modes
are reported and preserved. The installer never executes your profile to inspect
it. Malformed/edited/duplicate ownership markers are refused for manual review.

Install and Remove are idempotent and preserve unrelated bytes, including UTF-8
BOM, UTF-16 BOM, CRLF, and missing final newline. Existing files must be valid
UTF-8 or BOM-marked UTF-16. Reparse points/symlinks are refused. Backups use a
unique `.terminalhud-<id>.bak` beside the profile and are retained for you to
inspect. The small owned source block resolves the canonical installer receipt's
`active_build` on each new shell, so versioned upgrades need no profile rewrite.
The Windows client resolves it again for each request. Old package versions
remain subject to the installer's explicit session-aware cleanup policy.

## Validation

From the repository root:

```bash
python3 integrations/shells/tests/bash_pty.py
```

On native Windows, run each script in a separate no-profile PowerShell process:

```powershell
powershell -NoProfile -File integrations/shells/tests/profiles.tests.ps1
powershell -NoProfile -File integrations/shells/tests/line_editor.tests.ps1
powershell -NoProfile -File integrations/shells/tests/native_binding.tests.ps1
```

Repeat with `pwsh`. The first suite uses temporary files, the second uses a fake
line editor, and the third checks real PSReadLine binding metadata without
reading keyboard input. CI runs both Windows hosts. Bash tests own their PTYs,
use a fake Windows command, and disable history persistence. No test installs
hooks into a real profile or injects desktop input. A real HUD transition and
interactive Git Bash/WSL interoperability still require an explicit Windows
operator check.

The design is in [shell integration](../../docs/shell-integration.md). Prefix
handling follows [GNU Readline's key sequence timeout](https://www.gnu.org/software/bash/manual/html_node/Readline-Init-File-Syntax.html).
The [PSReadLine bindings](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/KeyBindings.cs)
use Escape as an Emacs chord prefix; its
[completion loop](https://github.com/PowerShell/PSReadLine/blob/master/PSReadLine/Completion.cs)
consumes Escape before dispatching ordinary handlers.
