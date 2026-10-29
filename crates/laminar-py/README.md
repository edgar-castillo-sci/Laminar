# laminar (Python bindings)

Python bindings for [Laminar](https://github.com/edgar-castillo-sci/Laminar),
a Rust suite for generating, canonicalizing, and exporting stacking
configurations of honeycomb layered materials.

**Status:** alpha (`0.1.0-alpha.4`). API may change.

## Installation (development)

The bindings are built with [maturin](https://www.maturin.rs/). From
`crates/laminar-py/`:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest
maturin develop --release
```

`maturin develop` compiles the `laminar` module with `--features python`
and installs it into the active virtual environment.

## Quick start

```python
import laminar

# 1. Count canonical classes at level n
count = laminar.sandwich(n=6, option="PL", count=True)
print(count)  # 70

# 2. List all canonical chains
chains = laminar.sandwich(n=2, option="OneH", export="stdout")
print(len(chains))  # 5
for c in chains:
    print(c)

# 3. Compare chains for equivalence
result = laminar.compare(["+0+j+j", "+j+0+j"], option="PL")
print(result["equivalent"])    # True
print(result["canonical_keys"])
print(result["orbits"])

# 4. Inspect a JamChain
chain = laminar.JamChain.parse("+j+0+j", option="PL")
print(chain.raw)        # "+j+0+j"
print(chain.canonical)  # "+0+j+j"
print(chain.k)          # 1
print(chain.cell)       # "PP"
print(chain.period)     # 1

# 5. Export to CIF
cell = laminar.ReferenceCell(
    a=3.16,
    c=12.30,
    z_up=1.56,
    z_down=-1.56,
    species_map={"j": "Mo", "k": "S"},
)
chain = laminar.JamChain.parse("*j+0(+k-k)", option="OneH")
cif = chain.to_cif(cell)
print(cif)
```

## API reference

### Module functions

| Function | Description |
|---|---|
| `laminar.sandwich(n, option, export="stdout", count=False)` | Generate canonical chains, or return the class count if `count=True`. |
| `laminar.compare(chains, option)` | Compare chains for equivalence. Returns a dict with `equivalent`, `canonical_keys`, `orbits`. |
| `laminar.modules()` | List available symmetry, notation, and export modules. |
| `laminar.version()` | Return the crate version (same as `laminar.__version__`). |

#### `sandwich(n, option, export="stdout", count=False)`

| Parameter | Type | Description |
|---|---|---|
| `n` | `int` | Number of layers (>= 1). |
| `option` | `str` | `"PL"`, `"BPL"`, `"BU"`, `"BBU"`, `"OneH"`, `"OneT"`, `"Janus1H"`, `"Janus1T"`. |
| `export` | `str` | Only `"stdout"` is supported in Python (returns `list[str]`). For other formats use the CLI. |
| `count` | `bool` | If `True`, returns `int` instead of `list[str]`. |

Returns `list[str]` (canonical chains) or `int` (class count).

#### `compare(chains, option)`

| Parameter | Type | Description |
|---|---|---|
| `chains` | `list[str]` | JAM chains to compare. |
| `option` | `str` | JAM option. |

Returns `dict`:
```python
{
    "equivalent": bool,
    "canonical_keys": dict[str, str],   # raw -> canonical
    "orbits": dict[str, list[str]],      # canonical -> [raw, ...]
}
```

### `JamChain`

| Member | Type | Description |
|---|---|---|
| `JamChain.parse(s, option)` | classmethod | Parse a JAM string. |
| `.raw` | `str` | Original serialization. |
| `.canonical` | `str` | Canonical representative. |
| `.option` | `str` | The option used to parse. |
| `.k` | `int` | Number of layers. |
| `.cell` | `str` | Cell name (`"PP"`, `"Pp"`, `"pP"`, `"Ap"`). |
| `.is_palindromic` | `bool` | Whether the word is a strict palindrome. |
| `.is_periodic` | `bool` | Whether the word is periodic. |
| `.period` | `int \| None` | Minimal period, if periodic. |
| `.to_cif(cell)` | method | Return CIF string. Requires `ReferenceCell`. |
| `.to_ase(cell)` | method | Return `ase.Atoms`. Requires the `ase` Python package. |

### `ReferenceCell`

| Member | Type | Description |
|---|---|---|
| `ReferenceCell(a, c, z_up, z_down, species_map)` | constructor | Build a reference cell. |
| `ReferenceCell.from_toml(path)` | classmethod | Load from a TOML file. |
| `.a`, `.c`, `.z_up`, `.z_down` | `float` | Cell parameters. |
| `.species_map` | `dict[str, str]` | JAM letter -> chemical species. |

Example TOML:

```toml
a = 3.16
c = 12.30
z_up = 1.56
z_down = -1.56

[species_map]
j = "Mo"
k = "S"
```

## Integration with ASE

```python
from ase.io import write
import laminar

cell = laminar.ReferenceCell(
    a=3.16, c=12.30, z_up=1.56, z_down=-1.56,
    species_map={"j": "Mo", "k": "S"},
)
chain = laminar.JamChain.parse("*j+0(+k-k)/*j+0(+k-k)", option="OneH")
atoms = chain.to_ase(cell)
write("mos2_2h.cif", atoms)
```

`to_ase` requires the `ase` package. Install with `pip install ase`.

## Run tests

```bash
pytest python/tests/ -v
```

## Development notes

- The Rust bindings live in `src/bindings.rs`, gated behind the
  `python` Cargo feature. `cargo build --workspace` does **not** require
  Python; only `maturin develop --features python` does.
- If `maturin develop` fails with "unsupported Python version", bump
  `pyo3` in `Cargo.toml` to a newer release compatible with your
  Python (e.g., `0.24` or `0.25`).
- The current `sandwich` export only supports `"stdout"`. For `cif`,
  `jam`, and file output, use the CLI (`laminar sandwich -e cif ...`).

## License

Dual-licensed under MIT OR Apache-2.0. See `LICENSE-MIT` and
`LICENSE-APACHE` at the repository root.
