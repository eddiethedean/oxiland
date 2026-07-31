# Contributing to Oxiland

Oxiland is a multi-surface RDF project with a Rust library, Python package,
command-line tool, compatibility evidence, and published documentation.
Contributions are welcome.

Start with the [project charter](https://github.com/eddiethedean/oxiland/blob/main/docs/CHARTER.md),
then use the [documentation index](https://oxiland.readthedocs.io/en/latest/)
for roadmap and decisions. User-facing guides live under
[`docs/users/`](https://github.com/eddiethedean/oxiland/tree/main/docs/users);
do not treat planning ADRs as the product manual.

## Repository map

| Path | Purpose |
|---|---|
| `src/` | Public Rust facade and implementation |
| `tests/`, `examples/` | Rust integration evidence and runnable workflows |
| `python/` | Maturin/PyO3 Python package, stubs, tests, and examples |
| `crates/oxiland-cli/` | Command-line package and workflow tests |
| `docs/users/` | Task-oriented product documentation |
| `docs/evaluators/` | Positioning and migration guidance |
| `compatibility/` | Inventories, conformance fixtures, and oracle harnesses |
| `api/` | Curated public API snapshot |
| `docs/design/`, `docs/reports/` | Design and historical evidence archives |

Before editing, check `git status` and preserve unrelated local changes.

## Fast path (docs, bugs, small fixes)

You do **not** need an inventory ID for:

- documentation typo / clarity fixes under `docs/users/` or `README.md`;
- bug fixes with a failing regression test;
- CI or tooling fixes that do not change public API semantics.

For those PRs: describe the user-visible problem, add or adjust a test when
behavior changes, run the relevant local checks, and keep the diff focused.

## Before starting (compatibility work)

Choose work from the current roadmap milestone or identify:

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
mkdocs build --strict
```

For Python package changes (`python/`):

```console
cd python
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pytest pyright
maturin develop
pytest -q
pyright
python examples/quick_start.py
python examples/select.py
python examples/parse_serialize.py
python examples/persistent.py
maturin build --release
```

For CLI changes:

```console
cargo test -p oxiland-cli
cargo run -p oxiland-cli -- --help
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
[`docs/COMPATIBILITY.md`](https://github.com/eddiethedean/oxiland/blob/main/docs/COMPATIBILITY.md).
Do not weaken a fixture or replace a Redland result with a new golden output
until the difference is classified.

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
- `docs/milestones/` says how a release was or will be delivered;
- `docs/EXECUTION.md` defines the operating model and current work order;
- `docs/DECISIONS.md` records durable design choices;
- `docs/RISKS.md` tracks uncertainty and release threats;
- `docs/reports/` preserves evidence for completed milestones.

Plans must label future work as planned. Historical reports should not be
silently rewritten to describe later implementation.

User documentation should:

- start from a user task rather than implementation provenance;
- state prerequisites, return values, failure behavior, and resource costs;
- distinguish in-memory examples from persistent production workflows;
- include operational limits and unsupported behavior near the relevant task;
- use copyable examples that compile or clearly mark illustrative placeholders;
- link to authoritative API, support, security, and upgrade contracts;
- avoid claims such as “fast”, “complete”, or “compatible” without evidence and
  scope.

Run both the local-link checker and strict MkDocs build after navigation or
cross-document changes. For `python/README.md` or a crate README, also build the
artifact and inspect its packaged metadata.

## Pull request checklist

- [ ] Scope is linked to inventory IDs, a release gate, or a docs/bug fast-path note.
- [ ] Public behavior and non-goals are documented.
- [ ] Positive, boundary, and failure tests are present when behavior changes.
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
