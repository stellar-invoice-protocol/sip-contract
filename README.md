# Stellar Invoice Protocol

A Soroban smart contract (Rust) implementing a simple invoice lifecycle on Stellar (Soroban).

Features
- Create invoices with issuer, payer, amount, currency identifier, due date
- Pay invoices (partial and full payments supported)
- Automatic status transitions (Created, PartiallyPaid, Paid, Overdue, Cancelled)
- List invoices by address (both issuer and payer)
- Mark invoices overdue based on ledger timestamp
- Persistent storage for invoices and an instance counter
- Events emitted on create, pay, and status change

Important: This scaffold accepts a currency identifier (Symbol) but does not implement token transfer or multi-currency swapping. Token integration is a future enhancement.

Building

Install the Soroban toolchain and wasm target (see CONTRIBUTING.md). Then build the wasmbinary with:

- Using soroban CLI: `stellar contract build`
- Or direct Rust build: `cargo build --target wasm32-unknown-unknown --release`

Running tests

Run unit tests with:

```text
cargo test
```

Deploying to testnet

After building, deploy using the Stellar CLI / soroban tooling:

```text
stellar contract deploy --wasm target/wasm32-unknown-unknown/release/stellar-invoice-protocol.wasm --network testnet
```

(Adjust the path to your .wasm artifact if needed.)

Contract function reference

- create_invoice(env, issuer: Address, payer: Address, amount: i128, currency: Symbol, due_date: u64) -> u64
  - Creates a new invoice, returns invoice id (u64). Emits Invoice/Created event.

- pay_invoice(env, invoice_id: u64, payer: Address, amount: i128)
  - Payer pays the invoice. Updates paid_amount and transitions status to PartiallyPaid or Paid. Emits Invoice/Paid event. Panics on unauthorized payer, overpayment, or paying cancelled/paid invoice.

- get_invoice(env, invoice_id: u64) -> Invoice
  - Returns the Invoice struct for given id. Panics if invoice not found.

- cancel_invoice(env, invoice_id: u64, issuer: Address)
  - Cancels an unpaid invoice. Only the issuer may cancel. Panics if unauthorized or if invoice already has payment or is paid.

- list_invoices_by_address(env, address: Address) -> Vec<u64>
  - Returns invoice ids associated with the given address (issuer or payer).

- mark_overdue(env, invoice_id: u64)
  - Anyone may call this. If the ledger timestamp is past the invoice due_date and invoice is unpaid, status changes to Overdue and an event is emitted.

Notes
- Currency is represented as a Symbol (e.g., `Symbol::short("XLM")`) or any token contract Address can be stored in the currency field in future updates.
- This initial version focuses on invoice lifecycle and storage; token transfers are out of scope for this iteration.

## Security & Access Control

1. **Payer Verification**: Only the designated `payer` is authorized to make payments towards an invoice. Overpayments are guarded against at the contract level.
2. **Issuer Verification**: Only the `issuer` who created the invoice is authorized to cancel it. Cancelation is restricted to unpaid, un-cancelled, and un-expired invoices.
3. **Overdue Transitions**: The `mark_overdue` function is permissionless but strictly validates the ledger timestamp against the due date before updating the state.

