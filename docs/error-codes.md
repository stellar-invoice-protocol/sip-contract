# Error Codes

Errors are defined in `src/errors.rs` as a `#[contracterror]` enum. The Soroban
host encodes them as `u32` discriminant values in the transaction result.

| Variant | Code | When it fires |
|---|---|---|
| `InvoiceNotFound` | 1 | No invoice exists for the supplied id. |
| `UnauthorizedPayer` | 2 | The `payer` argument does not match the invoice's recorded payer. |
| `InvoiceNotPayable` | 3 | The invoice is in a terminal state (Paid or Cancelled) and cannot accept payment; also returned when trying to cancel an already-cancelled invoice, or `mark_overdue` on a non-active/non-overdue invoice. |
| `OverpaymentNotAllowed` | 4 | The payment amount would push `paid_amount` above the invoice `amount`. |
| `PaidAmountOverflow` | 5 | `paid_amount + amount` overflowed `i128`. (Requires astronomically large values; included for correctness.) |
| `OnlyIssuerCanCancel` | 6 | The `issuer` argument does not match the invoice's recorded issuer. |
| `CannotCancelPaidInvoice` | 7 | The invoice has been fully paid and its state is terminal. |
| `InvalidAmount` | 8 | The `amount` argument is zero or negative. |
| `InvalidDueDate` | 9 | The `due_date` argument is not strictly greater than the current ledger timestamp. |

## How to inspect errors from a client

Using the generated Rust test client:

```rust
let err = client.try_pay_invoice(&id, &payer, &0).unwrap_err();
assert_eq!(err, Ok(InvoiceError::InvalidAmount));
```

`try_*` methods return `Result<Result<T, InvoiceError>, soroban_sdk::InvokeError>`.
A typed contract error is `Ok(InvoiceError::X)` in the inner `Result`. A host-level
error (e.g. failed `require_auth`) is `Err(InvokeError::Contract(..))`.
