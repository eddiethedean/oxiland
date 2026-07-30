# Oxiland project plans

Status: active planning index

These documents define how Oxiland progresses from its current Rust facade to
verified Redland compatibility.

## Start here

1. [Roadmap](ROADMAP.md) defines the 0.x outcomes and evidence gates.
2. [Execution plan](EXECUTION.md) turns those outcomes into vertical slices and
   provides the current 0.1 backlog.
3. [Architecture](ARCHITECTURE.md) defines components and dependency direction.
4. [Compatibility plan](COMPATIBILITY.md) defines exactly what parity means.
5. [Verification plan](VERIFICATION.md) defines acceptable evidence.
6. [Decision log](DECISIONS.md) records choices that constrain future work.
7. [Risk register](RISKS.md) records threats, mitigations, and release blockers.

The live subsystem-level status remains in the root
[parity ledger](../PARITY.md). Plans describe intended work; the ledger records
only work that has landed and been verified. Milestone reports live under
[`reports/`](reports/), starting with [0.1](reports/0.1.md).

## Document authority

When documents answer different questions, use this order:

| Question | Authority |
|---|---|
| What exists and is verified now? | [Parity ledger](../PARITY.md) |
| What release comes next? | [Roadmap](ROADMAP.md) |
| How is work sliced and completed? | [Execution plan](EXECUTION.md) |
| Where does code belong? | [Architecture](ARCHITECTURE.md) |
| What does compatibility mean? | [Compatibility plan](COMPATIBILITY.md) |
| What evidence is sufficient? | [Verification plan](VERIFICATION.md) |
| Why was a durable choice made? | [Decision log](DECISIONS.md) |
| What could block the plan? | [Risk register](RISKS.md) |

## Maintenance rules

- Plans use `planned`, `in progress`, `blocked`, and `complete`.
- Only verified implementation is marked complete.
- Roadmap gates link to durable evidence before a milestone closes.
- Accepted deviations have an owner and review milestone.
- Material API or compatibility choices receive a decision record.
- New high-impact uncertainty enters the risk register.
- All relative links and Rust documentation build before release.
