# Error Codes

Errors are defined in [`src/errors.rs`](../src/errors.rs) as a `#[contracterror]`
enum. The Soroban host encodes each variant as its `u32` discriminant in the
transaction result, so callers can match on the numeric code as well as the
named variant.

## Reference table

| Variant | Code | Raised by | Exact condition |
|---|---|---|---|
| `InvoiceNotFound` | 1 | [`pay_invoice` L97](../src/contract.rs#L97), [`get_invoice` L136](../src/contract.rs#L136), [`cancel_invoice` L163](../src/contract.rs#L163), [`mark_overdue` L211](../src/contract.rs#L211) | `storage().persistent().get(Invoice(id))` returns `None` — no invoice with that id has ever been created, or it has been evicted from storage. |
| `UnauthorizedPayer` | 2 | [`pay_invoice` L100](../src/contract.rs#L100) | `invoice.payer != payer` — the address that authorized the call is not the designated payer recorded on the invoice at creation time. |
| `InvoiceNotPayable` | 3 | [`pay_invoice` L104](../src/contract.rs#L104), [`cancel_invoice` L176](../src/contract.rs#L176), [`mark_overdue` L219](../src/contract.rs#L219) | **pay_invoice**: `invoice.status` is `Paid` or `Cancelled`. **cancel_invoice**: `invoice.status` is already `Cancelled`. **mark_overdue**: `invoice.status` is not `Created` or `PartiallyPaid`, or `ledger().timestamp() <= invoice.due_date`. |
| `OverpaymentNotAllowed` | 4 | [`pay_invoice` L113](../src/contract.rs#L113) | `invoice.paid_amount + amount > invoice.amount` — the payment would push the running total above the invoiced amount. |
| `PaidAmountOverflow` | 5 | [`pay_invoice` L110](../src/contract.rs#L110) | `invoice.paid_amount.checked_add(amount)` returns `None` — the `i128` sum overflows. Requires values near `i128::MAX`; included for arithmetic correctness. |
| `OnlyIssuerCanCancel` | 6 | [`cancel_invoice` L166](../src/contract.rs#L166) | `invoice.issuer != issuer` — the address that authorized the call is not the issuer recorded on the invoice at creation time. |
| `CannotCancelPaidInvoice` | 7 | [`cancel_invoice` L170](../src/contract.rs#L170) | `invoice.status == Paid` — a fully paid invoice is in a terminal state and cannot be cancelled. |
| `InvalidAmount` | 8 | [`create_invoice` L40](../src/contract.rs#L40), [`pay_invoice` L93](../src/contract.rs#L93) | `amount <= 0` — the `amount` argument is zero or negative. Checked before any storage access. |
| `InvalidDueDate` | 9 | [`create_invoice` L43](../src/contract.rs#L43) | `due_date <= env.ledger().timestamp()` — the due date is not strictly in the future relative to the current ledger timestamp at the moment of the call. |

## Reading errors from a Rust test client

The `try_*` methods on the generated client return
`Result<Result<T, InvoiceError>, soroban_sdk::InvokeError>`:

- `Ok(Ok(value))` — call succeeded, `value` is the return value.
- `Ok(Err(InvoiceError::X))` — call reached the contract and returned a typed error.
- `Err(InvokeError::Contract(..))` — host-level failure before contract logic ran
  (e.g. `require_auth()` rejected the caller).

```rust
// Typed contract error — matches Ok(Err(...))
let err = client.try_pay_invoice(&id, &payer, &0).unwrap_err();
assert_eq!(err, Ok(InvoiceError::InvalidAmount));

// Host auth failure — matches Err(Err(...))
let err = client.try_cancel_invoice(&id, &impostor).unwrap_err();
assert!(matches!(err, Err(_)));  // require_auth rejected impostor
```

## Reading errors from the Stellar CLI

A contract error appears in the CLI output as a `ContractError` with the numeric
code. For example, `InvoiceNotFound` (code 1) surfaces as:

```
error: transaction simulation failed: host invocation failed
  ... ContractError(1)
```

Cross-reference the code against the table above to identify the variant.
