# Contributing

Laminar is currently in the **specification phase (v0.0.1)**. The API and
crate structure are defined, but implementations are planned for v0.2.0
onwards. See `ROADMAP.md` for milestones.

We welcome **feedback on the API design and architecture** via issues.
Code contributions will be accepted starting with v0.2.0.

## When code contributions open

By submitting a pull request, you certify that:

1. The contribution was created in whole or in part by you, and you have
   the right to submit it under the project's license.
2. The contribution is provided under the **MIT OR Apache-2.0** dual license.
3. You understand that the contribution will be publicly available and
   that the project's governance is founder-led.

This is the **Developer Certificate of Origin (DCO)** model, used by the
Linux kernel and many other open-source projects. You do **not** assign
copyright; you license your contribution under the project's license.

Sign off your commits with `git commit -s -m "your message"`.

## Coding standards

- Rust 2021 edition.
- `cargo fmt` and `cargo clippy` must pass.
- All new public APIs require tests and documentation.
- All new algorithms require a reference to the paper or derivation that
  justifies them.

## What we look for

- Correctness over speed.
- Clarity over cleverness.
- Reproducibility: every claim must be verifiable by a test or a benchmark.

## What we do not accept

- Contributions that change the license.
- Contributions that add dependencies with incompatible licenses.
- Contributions that rename the project or rebrand it.
- Contributions that claim authorship over the JAM notation.
