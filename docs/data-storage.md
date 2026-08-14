# Data Storage

## Key design

All storage keys use the `DataKey` enum defined in `src/storage.rs`:

```rust
#[contracttype]
pub enum DataKey {
    Counter,                    // instance storage
    Invoice(u64),               // persistent storage
    AddressIndex(Address),      // persistent storage
}
```

Using a typed enum (rather than raw `Symbol` or tuple keys) means:

- Key collisions between different entry types are structurally impossible.
- The storage layout is self-documenting.
- The Soroban SDK serialises each variant with a unique discriminant, so
  `Invoice(1)` and `AddressIndex(addr)` can never alias.

## Storage tiers

### Instance storage

`DataKey::Counter` lives in instance storage. The counter is read and written on
every `create_invoice` call and exists for the lifetime of the contract instance.
Instance storage is extended by `bump_instance()`, which is called at the top of
every contract entrypoint.

### Persistent storage

`DataKey::Invoice(id)` and `DataKey::AddressIndex(addr)` use persistent storage.
These entries outlive individual transactions and are individually TTL-managed.

## TTL strategy

Soroban charges rent by keeping entries alive up to a declared TTL. If a TTL
expires the entry is evicted and the data is lost. This contract extends TTL
on every read and write using the following constants (see `src/storage.rs`):

| Constant | Ledgers | Approx. wall-clock time |
|---|---|---|
| `TTL_THRESHOLD` | 518,400 | 30 days |
| `TTL_EXTEND_TO` | 1,555,200 | 90 days |

At ~5 seconds per ledger: 17,280 ledgers ≈ 1 day.

**Threshold** means: if the current TTL is below this value, extend it.
**Extend-to** means: set the new TTL to this value.

Every `store_invoice` and `load_invoice` call extends the TTL for that specific
`Invoice(id)` key. Every `push_invoice_to_address` and `get_invoices_for_address`
extends the `AddressIndex(addr)` key. `bump_instance` handles the instance entry.

This means any invoice that has been interacted with in the last 30 days will
automatically stay alive for at least another 60 days after that interaction.

## What is not covered

There is no mechanism to restore evicted entries. If an invoice entry is evicted
(because 90 days passed without any interaction and no one called a contract
function that touched that specific key), it is gone. For production use, off-chain
indexing of emitted events provides a durable record.
