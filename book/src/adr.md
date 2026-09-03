# Architecture Decision Records

The ADRs are the case law of this workspace — every structural decision is
recorded with its context and consequences. They live in
[`docs/adr/`](https://github.com/Industrial-Algebra/Lonis/tree/develop/docs/adr):

| ADR | Decision |
|---|---|
| [0001](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0001-block-contract.md) | The block contract; `invoke → Vec<Block>`; extraction direction |
| [0002](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0002-typed-block-payloads.md) | `Block<P: BlockPayload>`; erasure is topological |
| [0003](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0003-subprocess-tool-protocol.md) | The subprocess wire protocol (bounded, isolated) |
| [0004](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0004-vertical-payload-authoring.md) | `#[derive(BlockPayload)]` + the authoring guide |
| [0005](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0005-json-schema-emission.md) | Curated JSON Schemas + golden fixtures (with the issue #10 erratum) |
| [0006](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0006-subprocess-provider.md) | The provider surface; lonis-as-provider |
| [0007](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0007-canonicalization-policy.md) | Content-hash canonicalization (number normalization) |
| [0008](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0008-replay-verification.md) | `verify_replay` — replay verification helper |
| [0009](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0009-stream-mode.md) | Stream mode: sync pull core, async at the host |
| [0010](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0010-v0.2-additive-wire.md) | v0.2 additive wire: typed identity seam, verification tier, no-match contract |
| [0011](https://github.com/Industrial-Algebra/Lonis/blob/develop/docs/adr/0011-recall-refusal-declination-register.md) | Recall refusal — the declination register (the deletion rule is the only door in) |

The upstream constitution is the
[Anima Ecosystem Doctrine](https://github.com/Industrial-Algebra) §2.7
(block taxonomy) and §3 (Lonis as the Tools plane).
