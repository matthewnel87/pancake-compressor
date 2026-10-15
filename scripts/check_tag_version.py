#!/usr/bin/env python3
"""Fail release if tag/app versions diverge."""
import json
import re
import sys
from pathlib import Path

def main():
    if len(sys.argv) != 2:
        raise SystemExit('Usage: check_tag_version.py vX.Y.Z')
    tag = sys.argv[1]
    if not re.fullmatch(r'v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?', tag):
        raise SystemExit(f'Invalid semantic version tag: {tag}')
    version = tag[1:]
    for fp in ('package.json', 'src-tauri/tauri.conf.json'):
        cfg = json.loads(Path(fp).read_text())
        actual = cfg.get('version')
        if actual != version:
            raise SystemExit(f'{fp} has version {actual!r}, expected {version!r}')
    print(f'Version checks passed for {tag}')

if __name__ == '__main__':
    main()
