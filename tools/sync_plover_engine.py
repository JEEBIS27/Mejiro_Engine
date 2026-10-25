#!/usr/bin/env python3
"""Export the authoritative Rust sources for standalone Plover wheel builds.

Run again after core changes. Generated vendor files are never edited in Plover.
"""
import argparse
from pathlib import Path
import shutil

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("plover_root", type=Path)
args = parser.parse_args()
source = Path(__file__).resolve().parents[1]
target = args.plover_root.resolve() / "vendor" / "mejiro-engine"
files = [Path("Cargo.toml"), Path("Cargo.lock"), Path("LICENSE"),
         Path("native/Cargo.toml"), Path("native/mejiro.h")]
files += [p.relative_to(source) for folder in ("src", "native/src")
          for p in sorted((source / folder).rglob("*.rs"))]
for relative in files:
    destination = target / relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source / relative, destination)
print(f"Exported {len(files)} Rust source files to {target}")
