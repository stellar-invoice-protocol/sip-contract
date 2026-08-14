# Architecture

The contract is split into five modules, each with a single responsibility.

## Module layout

```
src/
  lib.rs        — crate root; re-exports public types; declares the test module
  contract.rs   — the #[contract] / #[contractimpl] entry-points (public API)
  types.rs      — Invoice struct and Status enum (#[contracttype])
  storage.rs    — all storage reads/writes and TTL management
  events.rs     — event-emit helpers (one function per event)
  errors.rs     — InvoiceError enum (#[contracterror])
```

## Why this split?

**contract.rs** contains only orchestration logic: validate inputs, call storage
helpers, call event helpers, return results. It never calls `env.storage()` directly.
This makes the auth and business-logic paths easy to read in isolation.

**storage.rs** owns every `env.storage()` call in the codebase. TTL extension lives
here too, so there is exactly one place to change if Stellar's ledger timing changes.
All public functions in `storage.rs` extend TTL on every access — callers do not need
to think about storage rent.

**events.rs** has one `emit_*` function per event. Each function documents its topic
tuple and data tuple in a doc-comment. Keeping events separate means the event schema
is visible at a glance without reading contract logic.

**errors.rs** is a single `#[contracterror]` enum. Using a typed enum (rather than
`Symbol` constants) means the Soroban host encodes errors as `u32` values, which are
stable, comparable, and testable via `try_*` client methods.

**types.rs** holds the on-chain data types. Keeping them separate avoids circular
imports and makes the storage layout obvious.

## Data flow for a typical operation

```
Client call
  → contract.rs  (validate args, require_auth)
      → storage.rs  (load / mutate / store + extend TTL)
      → events.rs   (publish event)
  → return Result<T, InvoiceError>
```
