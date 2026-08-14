# Access Control

Every function that acts on behalf of a named address calls `address.require_auth()`
as the very first statement, before any state is read or modified. This is enforced by
the Soroban host: if the transaction is not signed by the required key, the call aborts
before any storage is touched.

## Per-function breakdown

| Function | Who must authorize | Why |
|---|---|---|
| `create_invoice` | `issuer` | The issuer is asserting a debt exists and binding their identity to the invoice. |
| `pay_invoice` | `payer` | Only the designated payer may reduce the outstanding balance. |
| `cancel_invoice` | `issuer` | Cancellation is a unilateral act by the party who raised the invoice. |
| `get_invoice` | nobody | Read-only; no state change. |
| `list_invoices_by_address` | nobody | Read-only; no state change. |
| `mark_overdue` | nobody (permissionless) | The transition is purely time-driven: the ledger timestamp is consensus-determined and cannot be manipulated by the caller. Restricting the caller would add friction without adding security. |

## What `require_auth` enforces

`require_auth` checks that the Soroban invocation context contains a valid
authorization entry for the calling address and this specific contract invocation.
In practice this means:

- For a user wallet address: the transaction must be signed by the corresponding
  Ed25519 or ECDSA key.
- For a contract address: the calling contract must have approved the sub-invocation
  via `authorize_as_current_contract` or an equivalent mechanism.

The check is performed by the host, not by application code, so there is no way to
bypass it through a crafted argument.

## What `require_auth` does not enforce

- It does not check that the authorized address matches a specific role stored on
  the invoice. That role check (`invoice.issuer != issuer`, `invoice.payer != payer`)
  is a separate, explicit comparison performed after the auth check. Both are required.
- It does not prevent a third party from *submitting* the transaction on behalf of
  the authorized address — it only checks the signature set.
