# Discovery-substrate recommendations — from the discovery-vertical seat

**Author seat:** the Knopper identity-restoration sessions, which ran both
IA discovery engines against a foreign workspace (Knopper @ `b180b60`,
release deferred) and filed practitioner feedback with each:

- karpal-discovery 0.9.0 — `Industrial-Algebra/Karpal` PR #160
  (`docs/dev/discovery-feedback-2026-08-22-knopper.md`)
- amari-discovery 0.24.1 — Amari PR #256
  (`docs/development/discovery-feedback-2026-08-22-knopper.md`)
- combined consumer-side report: Knopper PR #12
  (`docs/research/2026-08-22-discovery-amari-karpal.md`)

This document projects those lessons onto **Lonis** — the substrate
karpal-discovery already depends on (`lonis-schema`/`lonis-core` 0.1.0,
behind its `lonis` feature) and that amari-discovery is scheduled to
convert to (protocol.rs deletes into lonis-schema per ADR-0001's
extraction direction).

**Scope discipline:** the PULSE 2026-08-27 audit
(`IA-documents/RESEARCH_REPORTS/PULSE_2026-08-27_Lonis.md`) owns the
ecosystem-wide findings — typed-attribution identity, adapter-seam
ownership (LonisBlock/LonisEmitter), the RFC 8785 cross-check,
housekeeping debt. This document does not re-litigate those; it covers
what the **discovery verticals specifically** need from the substrate,
before the amari conversion multiplies the cost of getting it wrong.

---

## 1. What the substrate already got right (verified from the consumer seat)

- **The extraction target is aimed correctly.** `ReplayProvenance` is a
  superset of amari-discovery's provenance; `SchemaVersion` is the
  namespaced string marker (`<name>/v<N>`) the conversion plan requires.
  The two hard invariants from the 2026-08-10 architecture decision are
  already honored in 0.1.0.
- **The dependency story went clean.** karpal-discovery moved from the
  pre-publication git pin (`rev 4aa23a6`) to registry 0.1.0 with no
  protocol change, and validates envelopes against the published JSON
  Schemas with golden fixtures (ADR-0005). Consumer-side conformance is
  cheap and it works.
- **The adoption shape is right.** The component crates out-download the
  facade — consumers pull schema/derive/core separately, which is what a
  contract library wants.
- **ToolContract already carries the operational triad** —
  `Determinism`, `SideEffects`, `Cost` — which is most of what a probe
  needs (see R5).

## 2. Recommendations

### R1. Recall is a contract concern, not a vertical implementation detail

karpal-discovery's dominant failure (four documented misses where the
query vocabulary appeared **verbatim in the concept's own summary** —
"get and put" vs the optic entry's "get/put-style access") is a layer
bug, and it will **recur when amari-discovery ports**, because each
vertical implements recall independently over the substrate. The
cheapest fix from the karpal seat (index summaries+aliases as recall
text) is cheapest only if it is done **once**.

Two options, not mutually exclusive:

1. **Contract-level recall metadata** (wire-compatible, additive):
   optional `aliases: Vec<String>` and/or `recall_text` on the
   ToolContract-adjacent surface a discovery tool exposes, so verticals
   can build recall without re-crawling their own catalogs.
2. **A shared `lonis-recall` crate** — summary+alias indexing, stemming,
   synonym awareness — embedded by both verticals. The canonical-phrase
   failure mode then gets fixed in one place and stays fixed.

If Lonis stays recall-agnostic on principle, that is a defensible
position — but it should be **recorded as decided** (an ADR line), so
the recurrence in the amari port is a choice rather than an accident.

### R2. Verification tier belongs in the contract layer

The ranked #1 amari suggestion — a two-tier catalog (curated +
auto-extracted, marked unverified) — is really a protocol question.
`ReplayProvenance` records *who/how* but nothing carries **curation
status or confidence**. A small additive field on the payload surface
discovery tools expose (e.g. `verification: Curated | AutoExtracted
{ source, extracted_at } | Probed { evidence_hash }`) would let every
vertical express exactly the gap that dominated the Knopper run: real
machinery (`SchubertCalculus`, `schubert_cell_of`, `CayleyTable`)
existed in code but uncatalogued — an auto-extracted tier would have
surfaced it, marked, instead of hiding it behind an empty search result.

This composes with replay verification (ADR-0008): a successful replay
is precisely the evidence that promotes an entry from `AutoExtracted`
to `Probed`/`Curated`.

### R3. Doctrine as first-class tool metadata

The karpal feedback proposes a **doctrine/patterns aspect**: doctrine =
maintainer positioning constraints that steer discovery ranking.
Worked example (real, from the Knopper seat): "the GA substrate is the
identity; never recommend routing around geometry." That rule, had it
been machine-readable at discovery time, would have flagged exactly the
failure the Rabbit Hole audit later caught in shipped code — geometry
stubbed to ceremony to make a perf number.

`ToolContract` carries description/determinism/side-effects/cost but
nothing that says "this tool is load-bearing for the consumer's
architecture." A `doctrine: Vec<String>` (or a richer typed form) on the
contract surface gives hosts a place to pass such constraints into
ranking. Note the free lunch: `BlockKind::Capability` already embeds a
full `ToolContract`, so doctrine metadata flows into capability blocks
without a new kind.

### R4. Patterns as a shared corpus, not per-vertical overlays

The other half of the patterns aspect: pattern = named mapping from a
foreign architecture shape to overlay concepts, with **syn-AST
detection signals** (e.g. "trait with associated types Msg/Model" →
Elm-architecture machine → Store+ComonadEnv). Both verticals need this
— karpal maps foreign workspaces onto overlay concepts; amari maps them
onto catalog concepts — and AST detection needs a parser neither
vertical should own twice. Lonis is the neutral substrate both already
depend on; a shared patterns corpus (crate, or registry convention) is
its natural home. Patterns are query-shaped (input side), so they are
**not** the `Capability` block kind (output side) — they want a first
class representation of their own, but it can start as a payload
convention and graduate.

### R5. Wire the probe loop — the substrate has every piece except the name

The amari feedback ranked "runnable probe validation" third; it is
cheapest implemented once, at the substrate. A probe is just: a
side-effect-free tool invocation (`SideEffects::None` or `ReadOnly`,
`Determinism::Deterministic`, `Cost` bounded — all already expressible)
whose `Result`/`Evidence` blocks feed the verification tier from R2.
Bounds (ADR-0003) already fence it. What is missing is the loop as a
**named contract pattern**: a helper in lonis-core (or a documented
convention) so a vertical can declare "this catalog entry is backed by
a probe" and get execution + evidence recording for free.

### R6. Sequencing note on the amari conversion vs v0.2

One paragraph, deferring to the PULSE: the typed-attribution seam
(identity typing, the Lonis↔Dominic co-design) is the v0.2 question and
the PULSE owns it. From the discovery seat, one request only —
**decouple it from the conversion**. The extraction
(amari-discovery's protocol.rs deleting into lonis-schema) is valuable
on the 0.1.0 wire shape alone; if it waits for typed identity, the
catalog gap persists on both engines for the duration. Convert on 0.1,
let v0.2 land the typed seam independently.

### R7. Empty-result behavior is a protocol smell, not a UI choice

amari's search returned empty arrays where code-aware fallbacks were
possible, and an agent cannot distinguish "nothing exists" from "my
phrasing was wrong" — grep outperformed search in the live run. If
discovery tools are a first-class Lonis vertical, the substrate should
define what a **no-match** must carry: nearest neighbors with scores,
or an explicit diagnostic ("matched 0 concepts; closest by token
overlap: …"). Cheap to specify now while there are exactly two
implementations to align; expensive after N.

## 3. Suggested sequencing (all additive, none block each other)

| Order | Item | Shape | Cost |
|-------|------|-------|------|
| 1 | R2 + R7 | additive contract fields / convention | small |
| 2 | R1 | `lonis-recall` crate or ADR'd refusal | medium |
| 3 | R5 | probe pattern helper in lonis-core | small-medium |
| 4 | R3 + R4 | doctrine field; patterns corpus | medium |

R2/R7 are natural v0.2 content alongside the identity seam; none of
them require waiting for it. The amari conversion (R6) should proceed
on 0.1 regardless.

---

*Filed from the Knopper identity-restoration sessions. Cross-references:
PULSE_2026-08-27_Lonis (ecosystem audit, owns the attribution/adapter
findings); ADR-0001/0005/0008 (extraction direction, schema emission,
replay verification).*
