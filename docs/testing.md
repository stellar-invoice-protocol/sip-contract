# Testing

## Running the tests

```bash
cargo test
```

This runs all unit tests in `src/test/`. No special toolchain or external services
are required — the Soroban SDK's `testutils` feature provides an in-process
simulated environment.

To see test output including `println!` statements:

```bash
cargo test -- --nocapture
```

To run a single test file:

```bash
cargo test create_invoice
```

## Test structure

Tests live in `src/test/` and are gated behind `#[cfg(test)]` via `src/lib.rs`.

| File | What it covers |
|---|---|
| `common.rs` | Shared helpers: `setup()`, `create_default_invoice()`, timestamp constants. |
| `create_invoice.rs` | Happy path creation, sequential ids, field correctness, address indexing, `InvalidAmount`, `InvalidDueDate`, unknown id lookup. |
| `pay_invoice.rs` | Full payment, partial payment, accumulated partials, overpayment rejection, wrong-payer rejection, zero/negative amount, unknown id, paying a paid invoice, paying a cancelled invoice. |
| `cancel_invoice.rs` | Cancel created, cancel partially-paid, cancel-after-payment rejection, double-cancel rejection, wrong-issuer rejection, unknown id. |
| `overdue.rs` | Overdue transition from Created, from PartiallyPaid, before due_date rejection, on paid invoice, on cancelled invoice, already-overdue rejection, unknown id. |
| `authorization.rs` | Explicit `require_auth` checks without `mock_all_auths` — verifies that missing or incorrect authorization causes a host-level error. |

## Error coverage

Every variant in `InvoiceError` is triggered by at least one test via the `try_*`
method, which asserts `Err(Ok(InvoiceError::X))`:

| Error | Triggered in |
|---|---|
| `InvoiceNotFound` | `create_invoice::get_invoice_unknown_id_returns_not_found`, `pay_invoice::pay_invoice_unknown_id_returns_not_found`, `cancel_invoice::cancel_unknown_invoice_returns_not_found`, `overdue::mark_overdue_unknown_invoice_returns_not_found` |
| `UnauthorizedPayer` | `pay_invoice::pay_invoice_rejects_wrong_payer` |
| `InvoiceNotPayable` | `pay_invoice::cannot_pay_already_paid_invoice`, `pay_invoice::cannot_pay_cancelled_invoice`, `cancel_invoice::double_cancel_returns_error`, `overdue::mark_overdue_before_due_date_returns_error`, `overdue::mark_overdue_on_paid_invoice_returns_error`, `overdue::mark_overdue_already_overdue_returns_error` |
| `OverpaymentNotAllowed` | `pay_invoice::pay_invoice_rejects_overpayment`, `pay_invoice::pay_invoice_rejects_overpayment_after_partial` |
| `PaidAmountOverflow` | Not tested with real i128 overflow values — the checked_add guard is present; adding an overflow test requires i128::MAX inputs which is impractical in a unit test without mocking internals. |
| `OnlyIssuerCanCancel` | `cancel_invoice::cancel_by_wrong_issuer_returns_error` |
| `CannotCancelPaidInvoice` | `cancel_invoice::cancel_paid_invoice_returns_error` |
| `InvalidAmount` | `create_invoice::create_invoice_rejects_zero_amount`, `create_invoice::create_invoice_rejects_negative_amount`, `pay_invoice::pay_invoice_rejects_zero_amount`, `pay_invoice::pay_invoice_rejects_negative_amount` |
| `InvalidDueDate` | `create_invoice::create_invoice_rejects_due_date_in_past`, `create_invoice::create_invoice_rejects_due_date_equal_to_now` |

## What is not covered yet

- `PaidAmountOverflow` does not have a dedicated test; it requires passing
  `paid_amount` near `i128::MAX` which is not easy to set up without internal
  state injection.
- No integration test against a real Stellar testnet node.
- No fuzz testing or property-based testing.
