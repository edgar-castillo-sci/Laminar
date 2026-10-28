#!/usr/bin/env bash
set -euo pipefail

echo "[skeleton] building..."
cargo build --release -p laminar-skeleton

echo "[skeleton] running..."
./target/release/skeleton

echo
echo "[skeleton] validating with pymatgen (if available)..."
if python3 -c "import pymatgen" 2>/dev/null; then
    python3 - <<'PY'
from pymatgen.core import Structure
s = Structure.from_file("out/MoS2_1H.cif")
print(f"  pymatgen OK: {s.composition}")
PY
else
    echo "  (pymatgen not installed; skipping)"
fi

echo "[skeleton] validating with ASE (if available)..."
if python3 -c "import ase" 2>/dev/null; then
    python3 - <<'PY'
from ase.io import read
a = read("out/MoS2_1H.cif")
print(f"  ASE OK: {a.get_chemical_formula()}")
PY
else
    echo "  (ase not installed; skipping)"
fi

echo "[skeleton] done."
