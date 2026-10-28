# Known discrepancies between Laminar and the published papers

## PL, n=6: 70 vs 66

**Paper.** Table 3 of *Orbits as Multisets* lists PL n=6 = 66
(both Burnside (B) and recursive (R) columns).

**Laminar.** `augment_to(6, JamOption::PL)` returns 70 canonical
classes.

**Independent calculation.** Burnside's lemma on X_6 / G with
G = S3 × Z/2:

| Class         | \|Fix\|       | Total |
|---------------|---------------|-------|
| e             | 3^6 = 729     | 729   |
| v (flip)      | 3^3 = 27      | 27    |
| τ (3 transpositions) | 1 each | 3     |
| τv (3 transpositions × flip) | 3^3 = 27 each | 81 |
| ω, ωv (3-cycles) | 0         | 0     |

Sum = 729 + 27 + 3 + 81 = 840.  840 / 12 = **70**.

**Verification.** `kappa_literal == kappa_fast` holds for all 729 PL
chains at k=6 (test `literal_and_fast_agree_pl_k6_exhaustive`). So
Laminar's count is not an artifact of `kappa_fast`.

**Conclusion.** The paper's value 66 appears to be a typo. We treat 70
as correct. If canonJAM (the reference Python implementation) is
installed, `experiments/parity_canonjam.py` will also cross-check
against it; if canonJAM returns 66, we will re-open this issue.
