#!/usr/bin/env python3
"""Replace MejiroIME's generated Engine vendor tree with this worktree."""

import argparse
from pathlib import Path
import shutil
import tempfile


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("ime_root", type=Path)
args = parser.parse_args()

source = Path(__file__).resolve().parents[1]
ime_root = args.ime_root.resolve()
vendor = ime_root / "vendor"
target = vendor / "engine"
vendor.mkdir(parents=True, exist_ok=True)

with tempfile.TemporaryDirectory(prefix=".engine.", dir=vendor) as temporary:
    staged = Path(temporary) / "engine"
    shutil.copytree(
        source,
        staged,
        ignore=shutil.ignore_patterns(".git", "target"),
    )
    if target.exists():
        shutil.rmtree(target)
    staged.rename(target)

print(f"Synced Engine worktree to {target}")
