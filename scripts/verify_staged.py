#!/usr/bin/env python3
"""Verify a single non-empty, appropriately named installer staged by matrix job."""
from pathlib import Path
files = list(Path('dist').glob('*'))
if len(files) != 1 or not files[0].is_file() or files[0].stat().st_size < 1024:
    raise SystemExit('Expected exactly one staged installer with nontrivial size')
if files[0].suffix not in {'.exe','.dmg'}:
    raise SystemExit(f'Unexpected installer extension: {files[0]}')
print('Staged installer present:', files[0])
