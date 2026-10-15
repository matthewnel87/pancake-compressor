#!/usr/bin/env python3
"""Validate expected artifact set and create sha256sum-compatible digest file."""
from pathlib import Path
import hashlib
import argparse

ap=argparse.ArgumentParser()
ap.add_argument('--directory', default='release-artifacts')
ap.add_argument('--expected', type=int, default=3)
args=ap.parse_args()
folder=Path(args.directory)
files=sorted(x for x in folder.iterdir() if x.is_file() and x.suffix in {'.exe', '.dmg'})
if len(files) != args.expected or len({x.suffix for x in files}) != 2:
    raise SystemExit(f'Expected {args.expected} Windows + Mac installer files, found: {[x.name for x in files]}')
if sum(x.suffix == '.exe' for x in files) != 1 or sum(x.suffix == '.dmg' for x in files) != 2:
    raise SystemExit('Must contain exactly one EXE and two DMGs')
expected_tokens=['windows_x64-setup.exe','macos_arm64.dmg','macos_x64.dmg']
for token in expected_tokens:
    if not any(x.name.endswith(token) for x in files):
        raise SystemExit(f'Missing {token}')
lines=[]
for f in files:
    h=hashlib.sha256()
    with f.open('rb') as stream:
        for chunk in iter(lambda: stream.read(2**20),b''):
            h.update(chunk)
    lines.append(f'{h.hexdigest()}  {f.name}')
(folder/'SHA256SUMS.txt').write_text('\n'.join(lines)+'\n')
print('\n'.join(lines))
