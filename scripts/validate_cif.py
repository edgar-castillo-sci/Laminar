#!/usr/bin/env python3
"""
Validate Laminar-generated CIFs with pymatgen and/or ASE.

Usage:
    python3 scripts/validate_cif.py <cif_path>

If neither pymatgen nor ASE is installed, exits 0 with a warning.
"""

import sys
from pathlib import Path


def validate_pymatgen(path: Path) -> bool:
    try:
        from pymatgen.core import Structure
    except ImportError:
        return None
    try:
        s = Structure.from_file(str(path))
        print(f"  pymatgen OK: {s.composition}, {len(s)} sites")
        return True
    except Exception as e:
        print(f"  pymatgen FAILED: {e}")
        return False


def validate_ase(path: Path) -> bool:
    try:
        from ase.io import read
    except ImportError:
        return None
    try:
        a = read(str(path))
        print(f"  ASE OK: {a.get_chemical_formula()}, {len(a)} atoms")
        return True
    except Exception as e:
        print(f"  ASE FAILED: {e}")
        return False


def main() -> int:
    if len(sys.argv) != 2:
        print(f"usage: {sys.argv[0]} <cif_path>")
        return 1
    path = Path(sys.argv[1])
    if not path.exists():
        print(f"file not found: {path}")
        return 1
    print(f"[validate] {path}")
    pg = validate_pymatgen(path)
    ase = validate_ase(path)
    if pg is None and ase is None:
        print("  (neither pymatgen nor ase installed; skipping)")
        return 0
    ok = (pg is not False) and (ase is not False)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
