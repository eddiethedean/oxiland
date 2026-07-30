# Oxiland project plans

Status: active planning index

These documents define the product boundary, current evidence, and the path from
Oxiland's current Rust facade to verified Redland compatibility.

## Start here

1. [Project charter](CHARTER.md) defines users, scope, non-goals, invariants,
   success measures, and the 1.0 promise.
2. [Roadmap](ROADMAP.md) defines the 0.x release outcomes and evidence gates.
3. [0.2 milestone plan](milestones/0.2.md) specifies the current release's work
   packages, decisions, test matrix, and exit checklist.
4. [Execution plan](EXECUTION.md) defines how work becomes a verified vertical
   slice and records the current work order.
5. [Architecture](ARCHITECTURE.md) defines components and dependency direction.
6. [Compatibility plan](COMPATIBILITY.md) defines exactly what parity means.
7. [Verification plan](VERIFICATION.md) defines acceptable evidence.
8. [Decision log](DECISIONS.md) records choices that constrain future work.
9. [Risk register](RISKS.md) records threats, responses, triggers, and release
   blockers.

The live subsystem-level status remains in the root
[parity ledger](../PARITY.md). Plans describe intended work; the ledger records
only work that has landed and been verified. Milestone reports live under
[`reports/`](reports/), starting with [0.1](reports/0.1.md). The
[0.1.0 release checklist](reports/0.1.0-release.md) covers publishing steps.

## Document authority

When documents answer different questions, use this order:

| Question | Authority |
|---|---|
| Who is Oxiland for and what does 1.0 promise? | [Project charter](CHARTER.md) |
| What exists and is verified now? | [Parity ledger](../PARITY.md) |
| What release comes next? | [Roadmap](ROADMAP.md) |
| What exactly must the active release deliver? | [0.2 milestone plan](milestones/0.2.md) |
| How is work sliced, ordered, and completed? | [Execution plan](EXECUTION.md) |
| Where does code belong? | [Architecture](ARCHITECTURE.md) |
| What does compatibility mean? | [Compatibility plan](COMPATIBILITY.md) |
| What evidence is sufficient? | [Verification plan](VERIFICATION.md) |
| Why was a durable choice made? | [Decision log](DECISIONS.md) |
| What could block the plan? | [Risk register](RISKS.md) |

## Maintenance rules

- Plans use `planned`, `in progress`, `blocked`, and `complete`.
- Only verified implementation is marked complete.
- The roadmap contains release outcomes; detailed work packages live in
  `milestones/` and must not be duplicated inconsistently.
- Roadmap gates link to durable evidence before a milestone closes.
- Accepted deviations have an owner and review milestone.
- Material API or compatibility choices receive a decision record.
- New high-impact uncertainty enters the risk register.
- Historical reports are amended only to correct errors and clearly label later
  notes; they are not rewritten as current status.
- Links to implementation evidence use repository-relative paths and stable test
  names where possible.
- All relative links and Rust documentation build before release.
