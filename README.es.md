# Laminar

Un workspace de Rust para la enumeración exacta, canonicalización, clasificación y exportación de configuraciones de apilamiento en materiales de capas hexagonales. Incluye CLI y bindings de Python.

[Read in English](README.md)

JAM es la notación; honeycomb es el módulo de simetría. Laminar usa JAM como notación por defecto y honeycomb como módulo de simetría por defecto (y, en v0.1, el único).

- Versión: `0.1.0-alpha.5`
- Licencia: MIT OR Apache-2.0
- Zenodo: [10.5281/zenodo.23753545](https://doi.org/10.5281/zenodo.23753545) (concepto: [10.5281/zenodo.23628779](https://doi.org/10.5281/zenodo.23628779))

## Alcance

| Ítem | v0.1 |
|---|---|
| Módulo de simetría | honeycomb |
| Notación | JAM (opciones 1–8: PL, BPL, BU, BBU, OneH, OneT, Janus1H, Janus1T) |
| Canonicalización | O(k), constante 2 (ruta rápida) |
| Aumentación | libre de isomorfos, un representante por órbita |
| Clasificación en celdas | cuatro celdas (estructural, basada en palabras) |
| Formatos de exportación | CIF, POSCAR, XYZ, TSV, JAM |
| Comandos de CLI | `sandwich`, `compare`, `modules` |
| Bindings de Python | PyO3 |
| Muestreo MCMC | especificación lista; implementación en progreso |
| Reticulos distintos de honeycomb | no implementados |

La arquitectura (trait `StackingMode`) permite módulos de simetría y notaciones adicionales, pero en v0.1 no existe ningún módulo adicional.

## Estado

| Componente | Estado |
|---|---|
| Parser (8 opciones) | listo |
| Canonicalización (`kappa_literal`, `kappa_fast`) | listo |
| `children`, `fiber` | listo |
| `augment_to` | listo |
| Clasificación en celdas | listo |
| `ReferenceCell`, exportación a CIF | listo |
| CLI (`sandwich`, `compare`, `modules`, config) | listo |
| Bindings de Python | listo |
| Muestreador MCMC | en progreso |
| CLI `sample` | planeado |
| Caso de estudio DFT | planeado |

## Inicio rápido

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
# Conteo exacto de clases canónicas en el nivel n
laminar sandwich -n 6 --option PL -q
# 70

# Listar las cadenas canónicas
laminar sandwich -n 4 --option OneH -e stdout

# Comprobar equivalencia
laminar compare "+0*j(+k-k)" "*j+0(+k-k)" --option OneH
# equivalent: true

# Exportar todas las clases a CIF
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

## Arquitectura

Tres conceptos, separados:

| Capa | Qué define | v0.1 |
|---|---|---|
| Simetría | El grupo que actúa sobre las cadenas | honeycomb |
| Notación | Cómo se codifica una cadena | JAM (opciones 1–8) |
| Exportación | Cómo se serializa una cadena | CIF, POSCAR, XYZ, TSV, JAM |

```text
Laminar/
├── crates/
│   ├── laminar-core/     # tipos, simetría, canonicalización, aumentación, celdas
│   ├── laminar-io/       # ReferenceCell, CIF, POSCAR, XYZ, TSV
│   ├── laminar-markov/   # muestreo (en progreso)
│   ├── laminar-cli/      # sandwich, compare, modules
│   └── laminar-py/       # bindings de PyO3
├── examples/
├── experiments/          # paridad vs tablas publicadas, validación con pymatgen
└── spec/                 # especificación congelada + CHANGELOG
```

## Validación

| Comprobación | Resultado |
|---|---|
| Tests unitarios e integración | 138 (121 Rust + 17 Python), 0 fallos |
| Conteos vs resultados publicados | 37/37 coinciden |
| Validación externa con pymatgen | 12/12 casos, sin sobreconteo |
| Canonicalización: literal vs rápida | exhaustiva en k=6, aleatoria hasta k=12 |
| Invariancia de la aumentación | 2160 comparaciones, 0 excepciones |
| Benchmarks | la canonicalización rápida es 4.63–5.27× más rápida que la literal |

Reproducir:

```bash
./scripts/check.sh
python experiments/parity_canonjam.py
python experiments/validate_vs_pymatgen.py
cd crates/laminar-py && source .venv/bin/activate && python -m pytest python/tests/ -v
```

## Hoja de ruta

| Fase | Descripción | Estado |
|---|---|---|
| 0.5–5 | walking skeleton, 8 opciones, CLI, Python, pymatgen | listo |
| 6 | muestreo sobre el espacio de órbitas | en progreso |
| 7 | CLI `sample` con diagnósticos | planeado |
| 8 | caso de estudio DFT | planeado |
| 9–10 | paper de software, crates.io, PyPI, arXiv | planeado |

Historial completo: [`spec/CHANGELOG.md`](spec/CHANGELOG.md).

## Citación

```
Castillo, E. (2026). Laminar: Infrastructure for laminar stacking
(Version 0.1.0-alpha.5) [Computer software].
https://doi.org/10.5281/zenodo.23753545
```

El DOI de concepto (`10.5281/zenodo.23628779`) apunta siempre a la última versión. El botón "Cite this repository" de GitHub genera BibTeX a partir de `CITATION.cff`.

Base teórica: manuscritos acompañantes en preparación. Los enlaces se añadirán aquí cuando estén disponibles públicamente.

## Licencia y gobernanza

Doble licencia MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`). Proyecto liderado por el fundador (`GOVERNANCE.md`); el nombre "Laminar" es una marca registrada (`TRADEMARK.md`). Contribuciones bajo DCO (`CONTRIBUTING.md`).
