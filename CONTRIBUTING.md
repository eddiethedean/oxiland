# Contributing to Oxiland

Oxiland is early-stage and compatibility-driven. Contributions are welcome,
especially when they turn one Redland workflow into a complete, tested,
documented vertical slice.

Start with the [project charter](docs/CHARTER.md), then use the
[documentation index](docs/index.md#contributors) for the current milestone and
applicable decisions. User-facing guides live under
[`docs/users/`](docs/users/); do not treat planning ADRs as the product manual.


## Before starting

Choose work from the current milestone plan or identify:

- the Redland inventory IDs affected, or the named release gate enabled;
- the observable behavior and unsupported cases;
- the public ownership, lifetime, error, and streaming implications;
- the tests or fixtures that will prove completion;
- any architecture decision or risk that blocks the work.

Open a design discussion before implementing a new public abstraction,
persistence promise, extension mechanism, or C-facing ownership rule.

## Vertical slices

A compatibility slice normally contains:

1. inventory mapping;
2. public Rust mapping and documentation;
3. implementation through the public API;
4. positive, boundary, and failure tests;
5. standards or Redland differential evidence where applicable;
6. parity-ledger, migration, and release-note updates.

Infrastructure changes may omit inventory IDs, but must name the release gate
they enable. Code without the required evidence is `implemented`, not
`verified`.

## Local checks

Run:

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
python3 scripts/check-inventory.py
python3 scripts/check-docs.py
scripts/generate-public-api.sh check
```

For Python package changes (`python/`):

```console
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pytest pyright
maturin develop
pytest
pyright
```

Also run milestone-specific conformance, differential, storage, sanitizer, or
packaging checks when your change affects them.

## Public API changes

Oxiland is in 0.x, but public changes are still controlled:

- document the user problem before adding a facade or trait;
- prefer the smallest complete workflow over speculative flexibility;
- make expensive or potentially unbounded behavior visible;
- return typed errors for unsupported capabilities;
- include migration notes for breaking changes;
- refresh `api/oxiland-public-api.txt` intentionally and review its diff;
- note that the public-API check is a curated owned-symbol allowlist (not a
  full rustdoc/`cargo public-api` rustdoc JSON diff);
- add an ADR when the project charter's change-control rules require one.

Do not add a public placeholder that accepts configuration and fails generically
at runtime.

## Compatibility evidence

Compatibility tests must state what they prove. Where output has irrelevant
variation, use the normalization rules in
[`docs/COMPATIBILITY.md`](docs/COMPATIBILITY.md). Do not weaken a fixture or
replace a Redland result with a new golden output until the difference is
classified.

Every new inventory entry needs:

- a stable ID and normalized Redland symbol;
- subsystem and current state;
- safe Rust mapping or explicit disposition;
- implementation and test references when applicable;
- notes for deviations, platform limits, or incomplete evidence.

## Documentation changes

Keep document roles separate:

- `PARITY.md` says what is verified now;
- `docs/ROADMAP.md` says what each release must achieve;
- `docs/milestones/` says how the active release will be delivered;
- `docs/EXECUTION.md` defines the operating model and current work order;
- `docs/DECISIONS.md` records durable design choices;
- `docs/RISKS.md` tracks uncertainty and release threats;
- `docs/reports/` preserves evidence for completed milestones.

Plans must label future work as planned. Historical reports should not be
silently rewritten to describe later implementation.

## Pull request checklist

- [ ] Scope is linked to inventory IDs or a release gate.
- [ ] Public behavior and non-goals are documented.
- [ ] Positive, boundary, and failure tests are present.
- [ ] Relevant feature and platform configurations are tested.
- [ ] API snapshot changes are intentional.
- [ ] Compatibility differences are classified and recorded.
- [ ] Parity, plans, decisions, risks, and changelog are updated where affected.
- [ ] Local checks pass.
- [ ] No unrelated generated or local files are included.

## Safety and data integrity

The main crate has `#![forbid(unsafe_code)]`. Do not weaken it. Future C ABI
work belongs in the separately audited crate described by the architecture
plan.

Storage changes need reopen and failure-path tests. Changes that could alter or
lose persistent data require a migration/recovery story and a decision record
before they become public behavior.

## Commit and review shape

Keep commits reviewable and explain the compatibility consequence, not only the
implementation. Generated inventory or API snapshots should be committed with
the change that required them. Reviewers should be able to trace:

```text
Redland item or release gate
  -> documented mapping
  -> public behavior
  -> implementation
  -> evidence
  -> parity/release status
```
