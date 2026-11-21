#!/usr/bin/env python3
"""Rebuild the bundled MiSansVF.ttf subset from the full font in git history.

    python3 upstream/resources/fonts/subset_font.py

The recovery draws its words in three languages, so the subset must hold
every string literal of the UI plus the whole of GB2312 as a fallback.
Strings set in a device's TOML (processor names, model names) are also read
at runtime, so those files are scanned too -- otherwise symbols such as the
registered mark in a Snapdragon name would be missing glyphs on screen.
The full font (19.6 MB) is not kept in the tree; it is read straight from
commit c11f10f, which is where the unsubsetted font lives.

Three things this script deliberately gets right, each learned the hard way:
  * Rust's backslash-newline line continuation splits string literals across
    lines; without removing it first, whole sentences of the licence note
    are never collected and their glyphs go missing.
  * 'unicode_escape' decodes UTF-8 bytes as Latin-1 and turns Chinese into
    mojibake; only Rust's simple escapes are unescaped here.
  * Never subset an already-subsetted font: glyphs are lost a second time.
    Always start from the full font. The space is a real glyph, too.
"""

import glob
import io
import os
import re
import subprocess
import sys
import tempfile

from fontTools.ttLib import TTFont

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, '..', '..', '..'))
OUT = os.path.join(HERE, 'MiSansVF.ttf')
FULL_IN_GIT = 'c11f10f:upstream/resources/fonts/MiSansVF.ttf'


def literals(src):
    """Every string literal of a Rust source, line continuations removed."""
    src = re.sub(r'\\\n[ \t]*', '', src)
    for lit in re.findall(r'"((?:[^"\\]|\\.)*)"', src):
        yield (lit.replace('\\n', '\n').replace('\\t', '\t').replace('\\r', '\r')
                  .replace('\\"', '"').replace('\\\\', '\\'))


def ui_chars():
    chars = set(chr(c) for c in range(0x20, 0x7f))  # printable ASCII, space included
    for pattern in ('upstream/crates/skyward-ui/src/**/*.rs',
                    'upstream/crates/skyward/src/**/*.rs',
                    'upstream/crates/skyward-system-init/src/**/*.rs'):
        for path in glob.glob(os.path.join(ROOT, pattern), recursive=True):
            with io.open(path, encoding='utf-8') as f:
                for lit in literals(f.read()):
                    chars |= set(lit)
    # Device config strings (e.g. the soc marketing name) reach the screen at
    # runtime, so collect their characters from the quoted TOML values too.
    for path in glob.glob(os.path.join(ROOT, 'upstream/devices/*.toml')):
        with io.open(path, encoding='utf-8') as f:
            for lit in re.findall(r'"((?:[^"\\]|\\.)*)"', f.read()):
                chars |= set(lit)
    chars -= {'\n', '\t', '\r'}
    for hi in range(0xA1, 0xF8):
        for lo in range(0xA1, 0xFF):
            try:
                chars.add(bytes([hi, lo]).decode('gb2312'))
            except UnicodeDecodeError:
                pass
    return chars


def cmap_of(path):
    cmap = set()
    for table in TTFont(path)['cmap'].tables:
        cmap |= set(table.cmap.keys())
    return cmap


def main():
    # The full font is restored from git rather than kept in the tree.
    with tempfile.NamedTemporaryFile(suffix='.ttf', delete=False) as f:
        f.write(subprocess.check_output(['git', 'show', FULL_IN_GIT], cwd=ROOT))
        full = f.name
    try:
        full_cmap = cmap_of(full)
        chars = sorted(c for c in ui_chars() if ord(c) in full_cmap)
        dropped = sorted(c for c in ui_chars() if ord(c) not in full_cmap)
        print('charset:', len(chars), 'chars; the full font itself lacks', [repr(c) for c in dropped])

        with tempfile.NamedTemporaryFile('w', suffix='.txt', delete=False,
                                         encoding='utf-8', newline='\n') as t:
            t.write(''.join(chars))
            text = t.name
        try:
            subprocess.run([sys.executable, '-m', 'fontTools.subset', full,
                            '--text-file=' + text,
                            '--output-file=' + OUT,
                            '--layout-features=*', '--glyph-names', '--notdef-outline',
                            '--recalc-bounds', '--recalc-average-width', '--name-IDs=*',
                            '--name-legacy', '--name-languages=*'], check=True)
        finally:
            os.unlink(text)

        missing = sorted(c for c in chars if ord(c) not in cmap_of(OUT))
        print('wrote', OUT, round(os.path.getsize(OUT) / 1048576, 2), 'MB')
        print('missing glyphs:', missing if missing else 'NONE')
        if missing or ord(' ') not in cmap_of(OUT):
            sys.exit(1)
    finally:
        os.unlink(full)


if __name__ == '__main__':
    main()
