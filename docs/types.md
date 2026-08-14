# Types

All on-chain types are defined in `src/types.rs` and annotated with `#[contracttype]`
so the Soroban SDK can serialise and deserialise them to/from ledger storage and XDR.

## `Status` enum

```rust
#[contracttype]
pub enum Status {
    Created,
    PartiallyPaid,
    Paid,
    Overdue,
    Cancelled,
}
```

| Variant | Meaning |
|---|---|
| `Created` | Invoice has been created; no payment received yet. |
| `PartiallyPaid` | At least one payment has been received but `paid_amount < amount`. |
| `Paid` | `paid_amount == amount`; terminal state. |
| `Overdue` | Ledger timestamp has passed `due_date` and the invoice is not yet fully paid. |
| `Cancelled` | Issuer cancelled the invoice; terminal state. |

Terminal states (`Paid`, `Cancelled`) cannot transition to any other state.

## `Invoice` struct

```rust
#[contracttype]
pub struct Invoice {
    pub id: u64,
    pub issuer: Address,
    pub payer: Address,
    pub amount: i128,
    pub currency: Symbol,
    pub due_date: u64,
    pub status: Status,
    pub created_at: u64,
    pub paid_amount: i128,
}
```

| Field | Type | Description |
|---|---|---|
| `id` | `u64` | Globally unique, sequentially assigned by the contract counter. Starts at 1. |
| `issuer` | `Address` | The party who created the invoice (creditor). |
| `payer` | `Address` | The designated party who must pay (debtor). |
| `amount` | `i128` | Total amount due, in the smallest unit of `currency`. Must be > 0. |
| `currency` | `Symbol` | An identifier for the currency or token (e.g. `XLM`). This contract does not perform token transfer — it is a record-keeping identifier only. |
| `due_date` | `u64` | Unix timestamp (seconds) by which payment is expected. Must be strictly after the ledger timestamp at creation time. |
| `status` | `Status` | Current lifecycle state. |
| `created_at` | `u64` | Ledger timestamp at creation time. |
| `paid_amount` | `i128` | Cumulative amount paid so far. Starts at 0. |

### Notes

- `amount` and `paid_amount` are `i128` to accommodate large-denomination token
  amounts without overflow in normal use. Overflow of `paid_amount` is explicitly
  checked and returns `PaidAmountOverflow`.
- `currency` is a `Symbol` (max ~9 ASCII characters). It is stored as a label only;
  the contract does not call any token contract. Token integration is out of scope for
  this version.
