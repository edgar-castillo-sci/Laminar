# Laminar
[Leer en español](README.es.md)

A Rust workspace for the exact enumeration, canonicalization, classification, and export of stacking configurations in layered materials. Ships with a CLI and Python bindings.

JAM is the notation; honeycomb is the symmetry module. Laminar uses JAM as its default notation and honeycomb as its default (and, in v0.1, only) symmetry module.

- Version: `0.1.0-alpha.5`
- License: MIT OR Apache-2.0
- Zenodo: [10.5281/zenodo.23753545](https://doi.org/10.5281/zenodo.23753545) (concept: [10.5281/zenodo.23628779](https://doi.org/10.5281/zenodo.23628779))

## Scope

| Item | v0.1 |
|---|---|
| Symmetry module | honeycomb |
| Notation | JAM (options 1–8: PL, BPL, BU, BBU, OneH, OneT, Janus1H, Janus1T) |
| Canonicalization | O(k), constant 2 (fast path) |
| Augmentation | isomorph-free, one representative per orbit |
| Cell classification | four cells (structural, word-based) |
| Export formats | CIF, POSCAR, XYZ, TSV, JAM |
| CLI commands | `sandwich`, `compare`, `modules` |
| Python bindings | PyO3 |
| MCMC sampling | spec done; implementation in progress |
| Lattices other than honeycomb | not implemented |

The architecture (`StackingMode` trait) allows additional symmetry modules and notations, but no such module exists in v0.1.

## Status

| Component | State |
|---|---|
| Parser (8 options) | done |
| Canonicalization (`kappa_literal`, `kappa_fast`) | done |
| `children`, `fiber` | done |
| `augment_to` | done |
| Cell classification | done |
| `ReferenceCell`, CIF export | done |
| CLI (`sandwich`, `compare`, `modules`, config) | done |
| Python bindings | done |
| MCMC sampler | in progress |
| `sample` CLI | planned |
| DFT case study | planned |

## Quick start

### Rust

```rust
use laminar_core::canon::kappa_fast;
use laminar_core::children::augment_to;
use laminar_core::{JamChain, JamOption};

let chain = JamChain::parse("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH)?;
let canonical = kappa_fast(&chain);
println!("canonical: {}", canonical.raw());

let bilayers = augment_to(2, JamOption::OneH);
assert_eq!(bilayers.len(), 5);
```

### CLI

```bash
# Exact count of canonical classes at level n
laminar sandwich -n 6 --option PL -q
# 70

# List canonical chains
laminar sandwich -n 4 --option OneH -e stdout

# Equivalence check
laminar compare "+0*j(+k-k)" "*j+0(+k-k)" --option OneH
# equivalent: true

# Export every class to CIF
laminar sandwich -n 4 --option OneH -e cif \
    --output out/ --reference-cell examples/MoS2_1H.toml
```

### Python

```python
import laminar

laminar.sandwich(n=6, option="PL", count=True)   # 70
chains = laminar.sandwich(n=2, option="OneH", export="stdout")

result = laminar.compare(["+0+j+j", "+j+0+j"], option="PL")
print(result["equivalent"])                      # True

from ase.io import write
cell = laminar.ReferenceCell(
    a=3.16, c=12.30, z_up=1.56, z_down=-1.56,
    species_map={"j": "Mo", "k": "S"},
)
chain = laminar.JamChain.parse("*j+0(+k-k)", option="OneH")
write("mos2.cif", chain.to_ase(cell))
```

## Architecture

Three concerns, separated:

| Layer | What it defines | v0.1 |
|---|---|---|
| Symmetry | The group acting on chains | honeycomb |
| Notation | How a chain is encoded | JAM (options 1–8) |
| Export | How a chain is serialized | CIF, POSCAR, XYZ, TSV, JAM |

```text
Laminar/
├── crates/
│   ├── laminar-core/     # types, symmetry, canonicalization, augmentation, cells
│   ├── laminar-io/       # ReferenceCell, CIF, POSCAR, XYZ, TSV
│   ├── laminar-markov/   # sampling (in progress)
│   ├── laminar-cli/      # sandwich, compare, modules
│   └── laminar-py/       # PyO3 bindings
├── examples/
├── experiments/          # parity vs published tables, pymatgen validation
└── spec/                 # frozen specification + CHANGELOG
```

## Validation

| Check | Result |
|---|---|
| Unit + integration tests | 138 (121 Rust + 17 Python), 0 failures |
| Counts vs published results | 37/37 match |
| pymatgen external validation | 12/12 cases, no over-counting |
| Canonicalization: literal vs fast | exhaustive at k=6, random up to k=12 |
| Augmentation invariance | 2160 comparisons, 0 exceptions |
| Benchmarks | fast canonicalization 4.63–5.27× faster than literal |

Reproduce:

```bash
./scripts/check.sh
python experiments/parity_canonjam.py
python experiments/validate_vs_pymatgen.py
cd crates/laminar-py && source .venv/bin/activate && python -m pytest python/tests/ -v
```

## Roadmap

| Phase | Description | State |
|---|---|---|
| 0.5–5 | walking skeleton, 8 options, CLI, Python, pymatgen | done |
| 6 | sampling over the orbit space | in progress |
| 7 | `sample` CLI with diagnostics | planned |
| 8 | DFT case study | planned |
| 9–10 | software paper, crates.io, PyPI, arXiv | planned |

Full history: [`spec/CHANGELOG.md`](spec/CHANGELOG.md).

## Citation

```
Castillo, E. (2026). Laminar: Infrastructure for laminar stacking
(Version 0.1.0-alpha.5) [Computer software].
https://doi.org/10.5281/zenodo.23753545
```

The concept DOI (`10.5281/zenodo.23628779`) always points to the latest version. GitHub's "Cite this repository" button generates BibTeX from `CITATION.cff`.

Theoretical foundation: companion manuscripts in preparation. Links will be added here once publicly available.

## License and governance

Dual-licensed under MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`). Founder-led project (`GOVERNANCE.md`); the name "Laminar" is a trademark (`TRADEMARK.md`). Contributions under DCO (`CONTRIBUTING.md`).
