"""Laminar Python quickstart.

Requires: `maturin develop --release` from `crates/laminar-py/`.
"""

import laminar

print(f"Laminar version: {laminar.__version__}")

# 1. Count canonical classes
print(f"\nPL n=6: {laminar.sandwich(n=6, option='PL', count=True)} classes")
print(f"OneH n=4: {laminar.sandwich(n=4, option='OneH', count=True)} classes")

# 2. List all chains at a small n
chains = laminar.sandwich(n=2, option="OneH", export="stdout")
print(f"\nOneH n=2, {len(chains)} canonical chains:")
for c in chains:
    print(f"  {c}")

# 3. Compare
result = laminar.compare(["+0+j+j", "+j+0+j"], option="PL")
print(f"\nEquivalent: {result['equivalent']}")
print(f"Orbits: {len(result['orbits'])}")

# 4. JamChain
chain = laminar.JamChain.parse("+j+0+j", option="PL")
print(f"\nRaw: {chain.raw}")
print(f"Canonical: {chain.canonical}")
print(f"k = {chain.k}, cell = {chain.cell}")

# 5. Export to CIF
cell = laminar.ReferenceCell(
    a=3.16, c=12.30, z_up=1.56, z_down=-1.56,
    species_map={"j": "Mo", "k": "S"},
)
chain = laminar.JamChain.parse("*j+0(+k-k)", option="OneH")
cif = chain.to_cif(cell)
print(f"\nCIF (first 5 lines):")
for line in cif.split("\n")[:5]:
    print(f"  {line}")
