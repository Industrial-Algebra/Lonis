# ADR: Recall Refusal — The Declination Register (R1)

**Status:** Accepted (2026-09-02)
**Responds to:** R1 in
[`docs/feedback/discovery-substrate-recommendations-2026-08-28.md`](../feedback/discovery-substrate-recommendations-2026-08-28.md),
and to the question raised in `RABBIT_HOLE_2026-09-01_Lonis.md` §4:
*"the substrate's neutrality is not a default state, it is a maintained
artifact."*

## Context

R1 offered a choice: recall as shared substrate (a `lonis-recall` crate),
or recall-agnostic on principle, "recorded as decided (an ADR line) so the
recurrence is a choice rather than an accident." The rabbit hole sharpened
the stakes: a substrate grows by absorbing what its verticals *duplicate*
(the ADR-0001 deletion rule, which has a natural stopping condition); a
framework grows by absorbing what its verticals *want* (which has none).
Where the line sits decides what Lonis becomes.

v0.2 (ADR-0010) already tested the line once: R2 and R7 landed as contract
types with explicit fences — no search implementation, no curation
machinery. This ADR generalizes that discipline and pins it.

## Decision

**Recall is a vertical concern. Lonis will not provide a `lonis-recall`
crate.** The stopping rule for the substrate is the deletion rule: absorb
what independent verticals duplicate; never absorb what they merely want.

The substrate supplies the *contracts recall results cite* — blocks,
`NoMatchDiagnostic`, `Verification` — and nothing on the recall side of
their use.

### The declination register

Neutrality is maintained, not defaulted. What Lonis declines, and what
would reopen each refusal:

| # | Declined | Why | Reopens when |
|---|----------|-----|--------------|
| 1 | **Recall machinery** (R1: dedup, context-window selection, ranking) | No vertical duplication exists; want without duplication is framework gravity | karpal *and* amari both carry materially duplicated recall code — the deletion rule fires |
| 2 | **Search implementation** behind R7 | R7's obligation is answerable *with* an empty `nearest` + diagnostic; ranking is each vertical's own theory | two verticals converge on one ranking algorithm worth deleting into the substrate |
| 3 | **Curation machinery** behind R2 | `Verification` is data, not process; promotion tooling belongs to whoever runs the replay that produces the evidence | a shared promoter emerges across verticals |
| 4 | **Patterns corpus** (R4) | Program-sized, not field-sized; fenced by its own recommendation | a payload convention two verticals duplicate — then the deletion rule, not want, moves it |
| 5 | **Async runtime** | Decided: ADR-0009 — sync by physics; async is the session orchestrator's property | (see ADR-0009; unchanged) |
| 6 | **Full RFC 8785 canonicalization** | Decided: ADR-0007's scoped refusal — the residual is documented; exotic float spellings are the producer's problem | a cross-producer hash mismatch materializes in the wild |

Rules of the register:

- **Refusals carry reopening conditions.** A refusal without one is dogma;
  with one, it is a maintained decision. Every entry names the evidence
  that would reopen it.
- **The deletion rule is the only door in.** Duplication — not demand,
  not elegance — moves concern from vertical to substrate.
- **Declining is a commit.** Each entry is an answer the ecosystem can
  build on: verticals own recall *knowing* Lonis will not grow one under
  them.

## Consequences

- R1 is decided by refusal; any future `lonis-recall` proposal must argue
  the reopening condition, not the merit of recall.
- Verticals (karpal, amari-discovery) own recall semantics and may cite
  Lonis contracts in their results — `NoMatchDiagnostic` and `Verification`
  exist precisely so recall-bearing answers stay contract-shaped without
  the substrate absorbing the machinery.
- The register is the topic the rabbit hole handed to the next convergence
  document: refusals 1–4 are Lonis's; other planes may keep their own.
- This ADR is docs-only: no code, no wire change, no version movement.

---

*Cross-references: ADR-0001 (extraction direction — the deletion rule this
register generalizes), ADR-0007 (scoped canonicalization refusal), ADR-0009
(sync by physics), ADR-0010 (the v0.2 fences this register names),
RABBIT_HOLE_2026-09-01_Lonis §4 (the question), PULSE_2026-08-27_Lonis §6
(adapter-first sequencing — unaffected).*
