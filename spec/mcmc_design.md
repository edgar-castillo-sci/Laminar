# MCMC design: sampling over X_k/G

**Status:** Fase 6 (spec). Implementation in Fase 6.2.
**Target:** uniform distribution over the canonical classes of X_k/G.
**Dependency:** `laminar-core` (κ, all_rows, Group, Stabilizer).

---

## 1. Overview

We want to sample canonical classes c ∈ X_k/G with probability 1/|X_k/G|.

The naive approach (chain on X_k/G directly) is mathematically
awkward: the proposal probabilities depend on orbit sizes, and computing
the normalization requires knowing |X_k/G| (which we do not have a priori,
except by enumeration, which defeats the purpose of MCMC).

The standard trick is to run a chain on X_k with target proportional to
the stabilizer size |Stab(x)| and a G-equivariant proposal. The projected
chain on X_k/G is then a well-defined Markov chain with uniform stationary
distribution. This is the approach taken here.

---

## 2. State space and target

- **Chain state:** x ∈ X_k (a valid JAM chain of length k with option ω).
- **Target on X_k:** π(x) ∝ |Stab(x)|, where Stab(x) = {g ∈ G : g·x = x}.
  This is the only choice of target on X_k that induces a uniform
  distribution on X_k/G (see §3).
- **Stationary on X_k/G:** π̄(c) = 1/|X_k/G| (uniform), for c ∈ X_k/G.

**Why |Stab(x)|?** Within an orbit c, all x ∈ c have conjugate stabilizers,
hence the same size |Stab(c)|. The orbit-stabilizer theorem gives
|c| · |Stab(c)| = |G| = 12. So if π(x) ∝ |Stab(x)|, then:

    π̄(c) = Σ_{x∈c} π(x) = |c| · |Stab(c)| / Z = |G| / Z,

where Z = Σ_x |Stab(x)|. Since |G| is constant, π̄ is uniform on X_k/G. □

---

## 3. The quotient trick: why we cannot run directly on X_k/G

Running a chain directly on X_k/G requires proposal probabilities of the
form q(c → c') = Σ_{x∈c, y∈c'} q(x, y) / |c|. These depend on |c| and |c'|,
which are computable (|c| = |G|/|Stab(c)|) but make the MH ratio messier.
More importantly, to define q(c → c') we need to lift c to a specific
representative x ∈ c, which is not canonical.

The G-equivariant approach on X_k avoids this: the chain on X_k has a
G-invariant target and G-equivariant proposal. Therefore, the projection
map κ: X_k → X_k/G is a **lumpable** function, and the projected chain is
a Markov chain on X_k/G. Its stationary distribution is uniform.

**Verification of lumpability.** Let P(x, y) be the MH transition kernel
on X_k. For g ∈ G, we have P(gx, gy) = P(x, y) (G-equivariance of the
chain). Therefore, for x, x' ∈ c and c' ∈ X_k/G:

    P(x, c') = Σ_{y ∈ c'} P(x, y) = Σ_{y ∈ c'} P(gx, gy) = P(gx, c') = P(x', c'),

for any g with gx = x'. So the projected kernel P̄(c, c') := P(x, c') is
well-defined (independent of the chosen x ∈ c), and the projected chain
is a Markov chain with uniform stationary. □

**Implementation consequence:** we run the chain on X_k. At each step,
we record κ(x_t) as the sample from X_k/G. Duplicates (visiting the same
orbit multiple times) are allowed; the chain still samples uniformly
over X_k/G.

---

## 4. Proposals

We use a mixture of three proposals. At each step, pick one:

| Proposal | Probability | G-equivariant? | Symmetric? |
|---|---|---|---|
| Row mutation | (1 - ε) · 1/2 | yes | yes |
| Row swap     | (1 - ε) · 1/2 | yes | yes |
| Independent sampler | ε | yes | yes |

with ε = 0.01 (default; configurable).

The independent sampler guarantees **irreducibility** even if the
row-mutation or row-swap graph happens to be disconnected on X_k (it
is not, per §7, but the fallback is cheap and robust).

### 4.1 Row mutation

At state x with rows r_1, ..., r_k:

1. Pick i ∈ [k] uniformly.
2. Pick r' from `all_rows(ω) \ {r_i}` uniformly (q - 1 choices, where
   q = |all_rows(ω)|).
3. Set y = (r_1, ..., r_{i-1}, r', r_{i+1}, ..., r_k).

**G-equivariance.** G acts on x by permuting columns (σ ∈ S3) and
optionally flipping sign and layer order (v). The set `all_rows(ω)` is
closed under G (verified for all 8 options: rows are defined as "all
distinct permutations of [...]" and sign maps the set to itself).
Therefore, if r_i → r' is a valid mutation, then (g·r_i) → (g·r') is
also a valid mutation, and the probabilities match:

    q(x → y) = 1/k · 1/(q - 1) = q(g·x → g·y).  ✓

**Symmetry.** q(x → y) = 1/(k(q-1)) = q(y → x).  ✓

### 4.2 Row swap

At state x with rows r_1, ..., r_k:

1. Pick (i, j) with i < j uniformly (k(k-1)/2 choices).
2. Set y = x with rows i and j swapped.

**G-equivariance.** Under σ (column permutation), the row swap is
unchanged. Under v = flip (reverse layer order), the pair (i, j) maps to
(k+1-j, k+1-i), which is also a valid pair, and the probability of
picking it is the same (uniform over all pairs).  ✓

**Symmetry.** q(x → y) = 2/(k(k-1)) = q(y → x).  ✓

### 4.3 Independent sampler (fallback)

At state x:

1. Construct y by sampling each row independently and uniformly from
   `all_rows(ω)`: y = (r'_1, ..., r'_k), each r'_i ~ U(all_rows(ω)).

**Uniform on X_k.** Each of the q^k chains is equally likely, so
q(x → y) = 1/q^k for all y. This is uniform on X_k.

**G-equivariance.** Uniform on X_k is trivially G-equivariant (any G-set
X has a unique G-invariant probability distribution, which is uniform).
✓

**Symmetry.** q(x → y) = 1/q^k = q(y → x).  ✓

**Role.** The independent sampler can move anywhere in X_k, so the chain
is irreducible regardless of whether the mutate/swap chain is.

---

## 5. Metropolis-Hastings

**Acceptance ratio.** For any proposal q (symmetric, G-equivariant) and
target π(x) ∝ |Stab(x)|:

    α(x, y) = min(1, (π(y) q(y, x)) / (π(x) q(x, y)))
            = min(1, |Stab(y)| / |Stab(x)|).  (since q is symmetric)

**Algorithm (one step):**

    function step(x, ω, ε, rng):
        if rng.random() < ε:
            y = independent_sample(ω, k, rng)
        elif rng.random() < 0.5:
            y = row_mutate(x, ω, rng)
        else:
            y = row_swap(x, rng)
        ratio = |Stab(y)| / |Stab(x)|
        if rng.random() < min(1, ratio):
            return y
        else:
            return x

**Acceptance rate.** Expected to be in [0.2, 0.5] for reasonable
ε. If acceptance is too high (> 0.6), the chain is mixing slowly (too
few moves). If too low (< 0.1), it's rejecting too much (proposal is
too aggressive). The diagnostics in §8 will report the acceptance rate.

**Simplification for the implementation.** |Stab(x)| is computed once
per step. For the acceptance, we only need the ratio |Stab(y)|/|Stab(x)|,
which can be computed as (1 + |non-trivial stabilizer elements of y|) /
(1 + |non-trivial stabilizer elements of x|). For the current x, we cache
the previous result (no recomputation needed).

---

## 6. Detailed balance

We verify detailed balance for the mixture proposal.

**Proposal:** q(x, y) = ε · q_ind(x, y) + (1-ε)/2 · q_mut(x, y) + (1-ε)/2 · q_swap(x, y).

Since q_ind, q_mut, q_swap are all symmetric (q(x, y) = q(y, x)), their
convex combination q is also symmetric.

**Claim:** π(x) q(x, y) = π(y) q(y, x) for all x, y ∈ X_k, where π(x) =
|Stab(x)| / Z.

*Proof.*

π(x) q(x, y) = |Stab(x)| · q(x, y)
             = |Stab(x)| · q(y, x)        (q symmetric)
             = (|Stab(x)| / |Stab(y)|) · |Stab(y)| · q(y, x)
             = π(y) q(y, x) · (π(x) / π(y)) ... 

Wait, this requires π(x)/π(y) = |Stab(x)|/|Stab(y)|, which is exactly
π(x) ∝ |Stab(x)|. But that's the definition. Let me redo.

π(x) q(x, y) = |Stab(x)| · q(x, y)  (unnormalized π)
π(y) q(y, x) = |Stab(y)| · q(x, y)  (since q symmetric)

For detailed balance, need |Stab(x)| · q(x, y) = |Stab(y)| · q(x, y)
for all x ≠ y with q(x, y) > 0. This is NOT true in general (only when
|Stab(x)| = |Stab(y)|).

**Correction.** The correct MH ratio is:

    α(x, y) = min(1, (π(y) q(y, x)) / (π(x) q(x, y)))

which gives:

    α(x, y) · π(x) q(x, y) = min(π(x) q(x, y), π(y) q(y, x))
    α(y, x) · π(y) q(y, x) = min(π(y) q(y, x), π(x) q(x, y))

These are equal (both = min(A, B)). So detailed balance holds with the
MH correction. □

**Consequence.** The acceptance α(x, y) = min(1, |Stab(y)|/|Stab(x)|)
ensures detailed balance for any symmetric proposal. The mixture is
symmetric, so this applies.

**Note on the G-invariance of α.** Since |Stab(g·x)| = |Stab(x)| for all
g ∈ G (stabilizers of conjugate elements have the same size),
α(gx, gy) = α(x, y). So the chain is G-equivariant. ✓

---

## 7. Connectivity

The chain is irreducible on X_k because the independent sampler can
reach any state with positive probability. Even without the independent
sampler, the mutate/swap chain is irreducible on X_k (the proof sketch
below), but the independent sampler guarantees it robustly.

**Claim (sketch).** For any option ω, the graph on X_k with edges
(x, row_mutate(x)) ∪ (x, row_swap(x)) is connected.

*Why.* Row mutation allows changing one row at a time to any other
valid row. Since any valid chain can be reached from any other by
changing one row at a time (change row k, then k-1, ..., then 1), the
mutation-only graph is connected on X_k. Row swap is not needed for
connectivity, only for faster mixing. □

**Note.** Connectivity of X_k implies connectivity of the projected chain
on X_k/G (since κ is surjective onto X_k/G). So the chain samples the
full orbit space.

---

## 8. Diagnostics

The sampler reports (per run):

| Metric | Definition | Healthy range |
|---|---|---|
| `acceptance_rate` | #accepted / #steps | [0.2, 0.5] |
| `cell_distribution` | Fraction of samples in each cell (PP, Pp, pP, Ap) | depends on k; compare to exact |
| `stabilizer_distribution` | Fraction of samples in each |Stab| class | {1, 2, 3, 4, 6, 12} |
| `mixing_time_estimate` | Integrated autocorrelation time | depends on k; report only |
| `n_unique_orbits` | #distinct κ(x) visited / #steps | grows with steps |

**`mixing_time_estimate`.** Estimate via the initial positive sequence
of the autocorrelation function of the cell indicator (e.g., 1[x ∈ PP]).
If autocorrelation does not decay within the run, report "not mixed".

**Diagnostics emission.** Via `--diagnostics` flag on the CLI, or via
`sample(..., diagnostics=True)` in Python. Output: JSON (structured) or
human-readable text.

---

## 9. Validation strategy

For small k (k ≤ 8), we can compute the exact stationary distribution on
X_k/G by enumeration (via `augment_to(k, ω)`). Then we can:

1. **Distributional check.** Run the MCMC for N steps, compute the
   empirical distribution of cells (PP, Pp, pP, Ap), and compare to the
   exact distribution. Accept if L1 error < 5%.

2. **Uniformity check.** Run the MCMC, project to X_k/G, and check that
   every orbit is visited with approximately equal frequency. Accept if
   max deviation < 5% (or L∞ < 10%).

3. **Detailed balance check.** For a small state space (k ≤ 4), verify
   by exhaustive enumeration that π(x) P(x, y) = π(y) P(y, x) for all
   x, y ∈ X_k. This is a test, not a runtime check.

4. **Forbidden transitions.** The Paper 2 §8 lists transitions that
   should never occur (PP → Pp, Pp → PP, Pp → Pp) for k ≤ 7. Verify that
   the MCMC observes 0 such transitions in N steps. This is a strong
   consistency check.

5. **Stabilizer distribution.** For each option and k, compare the
   empirical distribution of |Stab(x)| to the exact one (from
   enumeration). Accept if L1 < 5%.

If all 5 checks pass for k = 6 (all 8 options) and k = 8 (PL, BU, OneH),
the sampler is validated.

---

## 10. Prerequisites (known issues to fix)

### P1: `Stabilizer::size()` is incorrect for |Stab| ∈ {6, 12}

Current code (in `crates/laminar-core/src/types.rs`):

```rust
impl Stabilizer {
    pub fn size(&self) -> usize {
        match self {
            Stabilizer::Trivial => 1,
            Stabilizer::Z2 => 2,
            Stabilizer::Z3 => 3,
            Stabilizer::Other(gens) => {
                let _ = gens;
                4  // <- placeholder, WRONG for |gens| >= 3
            }
        }
    }
}
```

For `gens.len() = 5` (|Stab| = 6) or `gens.len() = 11` (|Stab| = 12),
`size()` returns 4. This breaks the MH ratio.

**Fix:** `Stabilizer::Other(gens) => gens.len() + 1`. Or better, change
the API to compute size directly:

```rust
impl Stabilizer {
    pub fn size(&self) -> usize {
        match self {
            Stabilizer::Trivial => 1,
            Stabilizer::Z2 => 2,
            Stabilizer::Z3 => 3,
            Stabilizer::Other(gens) => 1 + gens.len(),
        }
    }
}
```

**Verification:** add a test that computes `size()` and compares to
`Group::elements().iter().filter(|g| apply(g, &x) == x).count()` for
random chains. Should be exact.

### P2: `Group::stabilizer` classification is misleading

Current code classifies by `gens.len()`:
- 0 → Trivial (|Stab| = 1)
- 1 → Z2 (|Stab| = 2)
- 2 → Z3 (|Stab| = 3)
- other → Other (|Stab| = 4, 6, or 12, but currently returns 4)

For `gens.len() = 5` (|Stab| = 6), the group could be D3 ≅ S3. For
`gens.len() = 11` (|Stab| = 12), it's the full G. The classification
Z2/Z3 for gens.len()=1,2 is correct (as argued in §2), but the naming
for the "Other" case is uninformative.

**Fix (low priority):** rename `Stabilizer::Other` to `Stabilizer::Custom`
or add a `Stabilizer::full()` variant. Not blocking; the size is what
matters for MCMC.

---

## 11. Implementation plan

### 11.1 `laminar-markov` crate

```
crates/laminar-markov/
├── Cargo.toml
└── src/
    ├── lib.rs          # sample(), MarkovChain
    ├── transitions.rs  # row_mutate, row_swap, independent_sample
    └── diagnostics.rs  # Diagnostics struct, mixing time estimate
```

### 11.2 Public API

```rust
pub struct MarkovChain {
    x: JamChain,           // current state
    option: JamOption,
    eps: f64,              // fallback probability
    rng: StdRng,
    stab_x: usize,         // cached |Stab(x)|
}

impl MarkovChain {
    pub fn new(option: JamOption, k: usize, eps: f64, seed: u64) -> Self;
    pub fn step(&mut self) -> ();
    pub fn current(&self) -> &JamChain;
    pub fn canonical(&self) -> JamChain;  // kappa_fast(current)
    pub fn reset(&mut self, x: JamChain);
}

pub struct SampleConfig {
    pub n: usize,
    pub option: JamOption,
    pub steps: usize,
    pub burn_in: usize,
    pub thin: usize,
    pub eps: f64,
    pub seed: u64,
}

pub struct SampleResult {
    pub samples: Vec<JamChain>,          // canonical classes
    pub diagnostics: Diagnostics,
}

pub fn sample(config: SampleConfig) -> SampleResult;
```

### 11.3 Testing (Fase 6.4)

- `step` preserves validity (all rows in rowset).
- `step` is G-equivariant: `kappa_fast(step(g·x)) == kappa_fast(step(x))` for all g ∈ G (statistical; use N random x).
- Detailed balance: exhaustive for k=2, PL, all x, y pairs (small).
- `sample` returns N/thin + 1 samples.
- Distributional checks (see §9): implemented in `experiments/mcmc_validate.py` (Python, using `laminar.sample`).

### 11.4 CLI integration (Fase 7)

```
laminar sample --n K --option OPT --steps N --burn-in M --seed S
                [--thin T] [--eps E] [--diagnostics] [--json]
                [-e jam|cif|stdout] [--output DIR] [--reference-cell FILE]
```

Output: same format as `sandwich` (chains or files), plus diagnostics
block if `--diagnostics`.

---

## 12. Open questions

- **Q1:** Should the target be uniform, or Boltzmann π(x) ∝ |Stab(x)| · exp(-βE(x))? For v0.1, uniform. For v0.2, add a `beta` parameter and an `energy: Option<fn(&JamChain) -> f64>` hook.
- **Q2:** Should we report the effective sample size (ESS) instead of / alongside the acceptance rate? Yes; ESS is more informative. Implementation: compute via autocorrelation (Geyer's initial positive sequence).
- **Q3:** Should the fallback be a mixture with the mutation/swap, or a separate "restart" (always accept a fresh sample every T steps)? Mixture is statistically cleaner (uniform stationary preserved); restart is simpler but wastes samples. For v0.1, mixture.
- **Q4:** Should the chain be parallelized (multiple independent chains in threads) or serial? Parallel chains (e.g., 4 chains × N/4 steps each) are ideal for diagnostics (Gelman-Rubin R̂). For v0.1, single chain; v0.2, `--chains N`.

---

**End of spec.** See `docs/bitacora.md` (once created) for Fase 6 progress.
