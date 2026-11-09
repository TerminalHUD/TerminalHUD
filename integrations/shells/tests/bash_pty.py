#!/usr/bin/env python3
"""Owned PTY tests. Never read or write a user's shell startup/history files."""
import os
from pathlib import Path
import pty
import select
import shlex
import signal
import tempfile
import time
import unittest

SOURCE = Path(__file__).resolve().parents[1] / 'bash' / 'terminalhud.bash'


class Shell:
    def __init__(self, directory, before=''):
        self.log = directory / 'requests'
        self.log.write_text('')
        fake = directory / 'powershell.exe'
        fake.write_text('#!/bin/bash\nprintf "%s\\n" "$*" >> "$TER27_LOG"\n')
        fake.chmod(0o700)
        inputrc = directory / 'inputrc'
        inputrc.write_text('set editing-mode emacs\nset keyseq-timeout 80\n')
        rc = directory / 'rc'
        rc.write_text('HISTFILE=/dev/null\nPS1="OWNED> "\nPS2="MORE> "\n'
                      + before + '\nsource ' + shlex.quote(str(SOURCE)) + '\n')
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.environ.update(PATH=str(directory) + ':/usr/bin:/bin',
                              INPUTRC=str(inputrc), TER27_LOG=str(self.log),
                              TERM='xterm', LC_ALL='C')
            os.execv('/bin/bash', ['bash', '--noprofile', '--rcfile', str(rc), '-i'])
        self.read()

    def read(self, seconds=0.2):
        deadline = time.monotonic() + seconds
        data = b''
        while time.monotonic() < deadline:
            if select.select([self.fd], [], [], max(0, deadline - time.monotonic()))[0]:
                data += os.read(self.fd, 65536)
        return data

    def send(self, data):
        os.write(self.fd, data)
        return self.read()

    def count(self):
        return len(self.log.read_text().splitlines())

    def close(self):
        os.kill(self.pid, signal.SIGTERM)
        # Interactive Bash can ignore SIGTERM; use a child-local SIGKILL only.
        os.kill(self.pid, signal.SIGKILL)
        os.waitpid(self.pid, 0)
        os.close(self.fd)


class BashEscape(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='terminalhud-shell-pty-')
        self.directory = Path(self.temp.name)
        self.shell = Shell(self.directory)

    def tearDown(self):
        self.shell.close()
        self.temp.cleanup()

    def test_empty_nonempty_and_prefix_keys(self):
        output = self.shell.send(b'\x1b')
        self.assertNotIn(b'[1]', output, 'Background job announcement leaked into prompt')
        self.assertEqual(self.shell.count(), 1)
        request = self.shell.log.read_text()
        self.assertIn('-NoProfile -NonInteractive -File', request)
        self.assertNotIn(' run', request)
        self.shell.send(b'abc\x1b')
        self.assertEqual(self.shell.count(), 1)
        self.shell.send(b'\x03')
        output = self.shell.send(b"printf '<%s>\\n' ac\x1b[Db\r")
        self.assertIn(b'<abc>', output)
        output = self.shell.send(b"printf '<%s>\\n' hello world\x1bb\x1bd\r")
        self.assertIn(b'<hello>', output)
        self.assertEqual(self.shell.count(), 1)

    def test_continuation_search_completion_and_program(self):
        self.shell.send(b'echo "\r')
        self.shell.send(b'\x1b')
        self.assertEqual(self.shell.count(), 0, 'PS2 is not an idle primary prompt')
        self.shell.send(b'\x03')
        self.shell.send(b'\x12')  # Readline owns Escape while incremental search runs.
        self.shell.send(b'\x1b')
        self.assertEqual(self.shell.count(), 0)
        self.shell.send(b'\x03')
        self.shell.send(b"bind '\"\\t\": menu-complete'\r")
        self.shell.send(b'printf\t\x1b')
        self.assertEqual(self.shell.count(), 0)
        self.shell.send(b'\x03')
        # A foreground program reads from this owned PTY, outside Readline.
        self.shell.send(b"python3 -c 'import tty,sys,os; tty.setraw(0); print(\"BYTE=%d\" % os.read(0,1)[0],flush=True)'\r")
        output = self.shell.send(b'\x1b')
        self.assertIn(b'BYTE=27', output)
        self.assertEqual(self.shell.count(), 0)

    def test_unknown_escape_sequences_never_request_passive(self):
        for sequence in (b'\x1bz', b'\x1b[999~', b'\x1b['):
            self.shell.send(sequence)
            self.assertEqual(self.shell.count(), 0)
            self.shell.send(b'\x03')

    def test_existing_longer_shell_command_and_macro_remain_bound(self):
        self.shell.close()
        self.shell = Shell(self.directory, "bind -x '\"\\ez\":printf CUSTOMLONG'\n"
                           "bind '\"\\eY\": \"macro_text\"'")
        self.assertIn(b'CUSTOMLONG', self.shell.send(b'\x1bz'))
        self.assertIn(b'macro_text', self.shell.send(b'\x1bY'))
        self.assertEqual(self.shell.count(), 0)

    def test_replaced_prompt_disables_requests(self):
        self.shell.send(b'PS1="CUSTOM> "\r')
        self.shell.send(b'\x1b')
        self.assertEqual(self.shell.count(), 0)

    def test_unsupported_modes_and_custom_bindings_preserved(self):
        for before, check in [
            ('set -o vi', b'[[ -o vi ]] && echo PRESERVED\r'),
            ("bind -x '\"\\e\":printf CUSTOM'", b'\x1b'),
            ('shopt -u promptvars', b'echo PRESERVED\r'),
        ]:
            self.shell.close()
            self.shell = Shell(self.directory, before)
            output = self.shell.send(check)
            self.assertIn(b'CUSTOM' if 'bind -x' in before else b'PRESERVED', output)
            self.assertEqual(self.shell.count(), 0)


if __name__ == '__main__':
    unittest.main()
