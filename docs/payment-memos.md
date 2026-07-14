# Invoice References

An invoice reference is a short, issuer-supplied identifier that links an
on-chain invoice to an off-chain record such as a purchase order, customer
account, or accounting-system entry.

References are optional. Existing clients can continue creating invoices
without one, while integrations that need reconciliation can attach one when
the invoice is created.

References are stored separately from `Invoice` records. This keeps the
serialized invoice shape stable for already-deployed contracts and allows the
feature to remain optional without adding placeholder data to every invoice.

## Validation

- A reference must contain at least one byte.
- A reference may contain at most 64 bytes.
- A reference is immutable after invoice creation.
- Only the invoice issuer may authorize a referenced invoice.
