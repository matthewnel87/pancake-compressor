#!/usr/bin/env python3
"""Stage exactly one Tauri installer as a stable, unambiguous artifact.

No signing or security claims; use with the build CI matrix only.
"""
from __future__ import annotations
import argparse
from pathlib import Path
import shutil
import sys

TARGETS = {
    'windows-x64': ('nsis', '.exe', 'windows_x64-setup.exe', 'x86_64-pc-windows-msvc'),
    'mac-arm64': ('dmg', '.dmg', 'macos_arm64.dmg', 'aarch64-apple-darwin'),
    'mac-intel': ('dmg', '.dmg', 'macos_x64.dmg', 'x86_64-apple-darwin'),
}

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--platform', required=True, choices=TARGETS)
    ap.add_argument('--version', required=True)
    args = ap.parse_args()
    bundle, suffix, output_suffix, triple = TARGETS[args.platform]
    # Cargo builds with workspaces may output under root/target or src-tauri/target.
    roots = (Path('target'), Path('src-tauri/target'))
    files: dict[str,Path] = {}
    for root in roots:
        if not root.exists():
            continue
        for candidate in root.rglob('*' + suffix):
            # Explicit --target CLI directs Tauri to target/<triple>/release/bundle/*.
            if candidate.parent.name == bundle and triple in candidate.parts and candidate.is_file() and candidate.stat().st_size > 0:
                files[str(candidate.resolve())] = candidate
    matches = list(files.values())
    if len(matches) != 1:
        print(f'ERROR: expected exactly 1 {bundle} {suffix}, got {len(matches)}:', matches, file=sys.stderr)
        return 1
    out = Path('dist')
    out.mkdir(exist_ok=True)
    dest = out / f'Pancake-Compressor_{args.version}_{output_suffix}'
    if dest.exists():
        raise RuntimeError(f'Unexpected release name collision: {dest}')
    shutil.copyfile(matches[0], dest)
    print(f'Staged {matches[0]} -> {dest} ({dest.stat().st_size} bytes)')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
