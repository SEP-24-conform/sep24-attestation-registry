# sep24-attestation-registry

A minimal Soroban smart contract that stores on-chain attestations of
[SEP-24](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0024.md)
conformance checks.

## Why this exists

[`sep24-conformance`](https://github.com/SEP-24-conform/sep24-conformance) can
tell you, right now, whether an anchor's `/info` endpoint matches the SEP-24
spec. But that result only exists wherever you happened to run the check. If
a wallet or another app wants to know "is this anchor currently verified?",
there's nothing to query — they'd have to trust a centralized list, or run
the check themselves.

This contract gives that result a durable, publicly queryable home. An
off-chain backend runs the actual conformance suite, and only if it passes,
submits a signed attestation here. Anyone — a wallet, a directory site, a
script — can then read `get_attestation(domain)` directly from the ledger
and get a tamper-evident answer, without trusting whoever runs the backend
to not lie about historical results after the fact.

This contract does **not** run checks itself and has no opinion on what
"conformant" means — it only records what it's told, gated by a single
admin key. The trust model is: trust the admin key to only submit real
results (verifiable, since the checker itself is open source), not trust
an opaque list.

## Interface

| Function | Auth required | Description |
|---|---|---|
| `initialize(admin: Address)` | — | One-time setup. Panics if already initialized. |
| `get_admin() -> Address` | — | Returns the current admin address. |
| `set_admin(new_admin: Address)` | current admin | Rotates the admin key. |
| `attest(domain: String, passed: bool, result_hash: BytesN<32>)` | admin | Records a conformance result for `domain`. Overwrites any prior attestation for the same domain. `result_hash` should be a hash (e.g. SHA-256) of the full conformance report, so a verifier can confirm a specific report matches this on-chain record. |
| `get_attestation(domain: String) -> Option<Attestation>` | — | Reads the latest attestation for `domain`, if one exists. |

```rust
pub struct Attestation {
    pub timestamp: u64,       // ledger close time when written
    pub passed: bool,
    pub result_hash: BytesN<32>,
}
```

## Deployed instances

| Network | Contract ID |
|---|---|
| Testnet | [`CAPBMA52MFEJQYF7IJCD3EK4NJERKMICYKVRF7JBEIHZI65Y2HA5SUR3`](https://stellar.expert/explorer/testnet/contract/CAPBMA52MFEJQYF7IJCD3EK4NJERKMICYKVRF7JBEIHZI65Y2HA5SUR3) |
| Mainnet | not yet deployed |

## Development

```sh
cargo test              # unit tests, including a negative auth test that
                         # verifies attest() actually rejects unauthorized calls
stellar contract build   # produces contracts/registry/target/wasm32v1-none/release/registry.wasm
```

To redeploy to testnet:

```sh
stellar keys generate registry-admin --network testnet --fund
stellar contract deploy \
  --wasm target/wasm32v1-none/release/registry.wasm \
  --source registry-admin --network testnet --alias registry
stellar contract invoke --id registry --source registry-admin --network testnet -- \
  initialize --admin "$(stellar keys address registry-admin)"
```

## What this is not

This is not a general-purpose anchor directory or reputation system. It
stores exactly one thing — the latest pass/fail attestation per domain from
one specific admin key — deliberately kept small so its behavior is easy to
audit in full.

## License

Apache-2.0
