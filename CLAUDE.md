# polkadot-runtimes-lite

Stripped-down fork of [polkadot-fellows/runtimes](https://github.com/polkadot-fellows/runtimes) containing only the **Polkadot relay chain runtime** and **Asset Hub Polkadot runtime**, plus the chain-spec-generator. Everything else has been removed to make the repo easier to update for the purpose of running benchmarks.

## Repository layout

```
├── Cargo.toml                        # Workspace root — ALL deps declared here
├── chain-spec-generator/             # CLI for generating chain specs
├── pallets/
│   ├── ah-ops/                       # Asset Hub operations pallet
│   └── rc-migrator/                  # Relay chain migrator pallet
├── relay/
│   ├── kusama/constants/             # Kusama constants (kept for upstream compat)
│   └── polkadot/                     # Polkadot relay chain runtime
│       ├── constants/                # Relay chain constants
│       └── src/
│           ├── lib.rs                # construct_runtime!, pallet configs
│           ├── xcm_config.rs         # XCM executor + router config
│           ├── governance/           # OpenGov origins + tracks
│           ├── weights/              # Auto-generated benchmark weights
│           └── genesis_config_presets.rs
└── system-parachains/
    ├── asset-hubs/
    │   └── asset-hub-polkadot/       # Asset Hub Polkadot runtime
    │       ├── src/
    │       │   ├── lib.rs            # construct_runtime!, pallet configs
    │       │   ├── xcm_config.rs     # XCM executor config
    │       │   ├── staking/          # Staking-related configs (NPoS, DAP, bags)
    │       │   ├── governance/       # Governance origins
    │       │   ├── treasury.rs       # Treasury + bounties config
    │       │   └── weights/          # Auto-generated benchmark weights
    │       └── primitives/           # Shared primitives
    ├── bridge-hubs/                  # Primitives only (kept for upstream compat)
    ├── collectives/                  # Constants only (kept for upstream compat)
    ├── common/                       # Shared types across system parachains
    └── constants/                    # Shared constants (async backing, currency, fees)
```

## Upstream relationship

- **origin**: `https://github.com/paritytech/polkadot-runtimes-lite` (this fork)
- **upstream**: `https://github.com/polkadot-fellows/runtimes` (canonical fellowship repo)

This repo is regularly rebased/merged with upstream. **Never remove or inline crates that exist in upstream**, even if they appear unused or nearly empty — doing so creates merge conflicts on every upstream sync. Crates like `relay/kusama/constants`, `system-parachains/bridge-hubs/*/primitives`, and `system-parachains/collectives/*/constants` exist solely to preserve structural compatibility.

## Dependencies

All polkadot-sdk dependencies are git deps pointing at a specific commit:

```
git = "https://github.com/paritytech/polkadot-sdk"
rev = "5a8eaa7825953ed960d88ff73dd6c363f59495d2"
```

**Every dependency must be declared in the workspace `Cargo.toml`** and inherited by member crates with `{ workspace = true }`. No crate may add a direct dependency without it being in the workspace table first. When updating the SDK revision, you must update every git dep to the same commit and then fix any API changes.

Non-SDK dependencies (hex, codec, serde, etc.) use versioned crates.io deps.

## Building and checking

```sh
# Check everything compiles (both runtimes + tests + benchmarks)
cargo check --all-targets

# Build the WASM runtimes (slow — compiles wasm32 targets)
cargo build --release

# Run tests
cargo test --all-targets

# Format check
cargo fmt -- --check

# Lint
cargo clippy --all-targets
```

Always run `cargo check --all-targets` before committing any structural change (dependency updates, crate removals, pallet config changes). Both runtimes compile as native and as WASM — the WASM build is triggered automatically by `substrate-wasm-builder` in the build scripts.

## Key architectural concepts

### Runtimes and FRAME

Both runtimes are built with [FRAME](https://docs.substrate.io/reference/frame-macros/), Substrate's framework for composing blockchain runtimes from modular pallets. The central macro is `construct_runtime!` in each `lib.rs`, which wires together all pallets with their assigned indices.

Each pallet has a `Config` trait that the runtime must implement. When the upstream SDK changes a `Config` trait (adds/removes associated types), the runtime impl must be updated to match.

### Async backing and elastic scaling

Asset Hub runs with **elastic scaling** — 3 cores, 2-second block times:

- `BLOCK_PROCESSING_VELOCITY = 3`
- `UNINCLUDED_SEGMENT_CAPACITY = 12` (derived as `(3 + RELAY_PARENT_OFFSET) * VELOCITY`)
- `RELAY_PARENT_OFFSET = 1`
- `SLOT_DURATION = 2000ms` (= 6000ms relay slot / 3 velocity)

These constants live in `system-parachains/constants/src/polkadot.rs` and `system-parachains/constants/src/async_backing.rs`. Changing them affects block production timing. Do not modify without understanding the implications.

### Weight files

Files in `*/weights/` are **auto-generated by benchmarks**. They implement `WeightInfo` traits defined in the upstream SDK pallets. When the SDK renames or adds weight functions, the weight files must be updated to match the new trait signatures. Use the SDK's reference implementations (e.g., in `substrate/frame/*/src/weights.rs`) as a guide for stub values when new functions are added.

### XCM configuration

`xcm_config.rs` in each runtime configures cross-consensus messaging. Key types:
- `XcmRouter` — how to send XCM messages (relay uses child parachain router, asset hub uses parent)
- `XcmConfig` — the xcm-executor config (asset transactors, barriers, weighers, asset traps)
- `AssetTrap` — must implement `TrapAndClaimAssets` (the old separate `AssetClaims` was removed)

### Staking (Asset Hub)

Asset Hub has async staking (NPoS migrated from the relay chain):
- `pallet_staking_async` — the core staking pallet
- `pallet_election_provider_multi_block` — multi-block election with signed/unsigned phases
- `pallet_dap` — Decentralized Autonomous Proposal / inflation drip pallet
- `pallet_bags_list` — voter list sorted by bags

The staking configuration is in `system-parachains/asset-hubs/asset-hub-polkadot/src/staking/mod.rs`.

### Treasury and payments

The relay chain treasury uses `PayOverXcm` to make payments via XCM to Asset Hub. The second type parameter of `PayOverXcm` is the **XcmConfig** (not the XcmRouter — this changed in recent SDK versions).

## Common SDK update patterns

When updating the polkadot-sdk git revision, expect these kinds of breaks:

| Pattern | What to do |
|---------|-----------|
| Missing trait items in pallet `Config` | Add the new associated types. Check SDK test mocks or reference runtimes (e.g., `rococo`, `asset-hub-westend`) for values |
| Weight function renames | Rename in the weight file to match the new `WeightInfo` trait |
| Weight function parameter changes | Update signature (often params are removed) |
| Removed trait items | Delete from the impl block |
| Derive macro deprecations | Replace (e.g., `RuntimeDebug` → `Debug`) |
| Moved modules | Update import paths (e.g., `assigner_coretime` → scheduler submodule) |
| New `Config` supertrait bounds | Implement the new required config (e.g., `PermitConfig`) |
| Session API version bumps | Update `generate_session_keys` signature |

Reference runtimes for comparison:
- Relay: `/polkadot/runtime/rococo/` or `/polkadot/runtime/westend/` in the SDK
- Asset Hub: `/cumulus/parachains/runtimes/assets/asset-hub-westend/` in the SDK

The SDK checkout after `cargo update` lives at:
`~/.cargo/git/checkouts/polkadot-sdk-*/` (the hash suffix varies).

## Formatting

- `max_width = 100` and `comment_width = 100` (see `.rustfmt.toml`)
- Hard tabs, Unix newlines
- Comments must be packed to the full width — don't leave short lines when more words could fit

## Git conventions

- Never reference Claude or AI assistance in commit messages, PR descriptions, or author fields
- Commit messages should be concise and describe *why*, not *what*
- Always `cargo check --all-targets` before committing
- Always `git push` before presenting work for review
- Act first, commit, then present — don't ask permission for routine changes
