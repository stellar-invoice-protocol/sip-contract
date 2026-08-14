# Security Considerations

> No formal security audit has been performed on this codebase. This document
> describes the security properties the design aims to provide and the areas
> a future audit should examine.

## Authentication model

Every mutating function that acts on behalf of a named address calls
`address.require_auth()` as its first statement. The Soroban host enforces this
check before any state is read or modified. See `docs/access-control.md` for the
per-function breakdown.

A separate role check follows the auth check in all cases. For example,
`pay_invoice` verifies both that `payer` authorized the call *and* that `payer`
matches the invoice's stored `payer` field. Passing one check but not the other
returns an appropriate error.

`mark_overdue` is intentionally permissionless. The transition condition
(`ledger().timestamp() > due_date`) is determined entirely by consensus — a caller
cannot manipulate it. Restricting who can call `mark_overdue` would add friction
without adding security.

## Arithmetic

All payment accumulation uses `checked_add`. Overflow returns
`InvoiceError::PaidAmountOverflow` rather than wrapping silently. The release
profile sets `overflow-checks = true`, which also catches overflow in debug builds.

## Input validation

- `amount` must be > 0 (checked in both `create_invoice` and `pay_invoice`).
- `due_date` must be strictly after the current ledger timestamp (checked in
  `create_invoice`).
- Overpayment is explicitly rejected: `paid_amount + amount > invoice.amount` returns
  `OverpaymentNotAllowed`.

## Storage expiry

Persistent entries have TTL extended on every access. An entry that is never accessed
for ~90 days will expire. Expired entries are lost; there is no restore path in the
contract. Off-chain event indexing is the appropriate durability mechanism.

## What a formal audit should examine

- **Auth ordering**: confirm `require_auth()` is always the first statement, before
  any storage reads, to prevent information leakage before the auth check.
- **Status machine completeness**: confirm all possible `Status` variants are handled
  (no unhandled arm that falls through to incorrect behavior).
- **Integer arithmetic**: review all `i128` arithmetic paths for edge cases, even
  though `checked_add` is used.
- **Storage key collisions**: verify `DataKey` variant serialisation does not alias
  between variants.
- **Event data correctness**: confirm event data matches the documented schema.
- **Denial of service**: assess whether an adversary can fill an address index
  (unbounded `Vec<u64>`) to exhaust storage or computation limits.

## Out of scope

This contract does not perform token transfers, hold funds, or interact with any
external contract. The attack surface is limited to the state machine described above.
