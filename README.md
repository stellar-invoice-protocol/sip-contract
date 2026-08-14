[![CI](https://github.com/stellar-invoice-protocol/stellar-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-invoice-protocol/stellar-contract/actions/workflows/ci.yml)

# Stellar Invoice Protocol

A Soroban smart contract (Rust) implementing an invoice lifecycle on Stellar.

## Project status

**Implemented in this version:**
- Create, pay, cancel, and mark-overdue lifecycle for invoices
- Partial payment support with accumulated `paid_amount`
- Persistent storage with TTL extension on every access
- Events on create, pay, cancel, and overdue transition
- `require_auth()` enforced on all mutating, identity-bound operations
- Full unit test suite covering all public functions and every error variant

**Not implemented (out of scope for this milestone):**
- Token transfer — the `currency` field is a record-keeping label only; no funds move
- Multi-currency swaps or oracle pricing
- Invoice references / purchase-order attachments

## Building

Install the Rust `wasm32-unknown-unknown` target and the Stellar CLI, then:

```bash
# Run unit tests (native)
cargo test

# Build the WASM binary
cargo build --target wasm32-unknown-unknown --release --locked
```

Or use the Makefile shortcuts (see `make help`):

```bash
make test    # run the test suite
make build   # compile the release WASM
make all     # fmt + lint + test + build in sequence
```

See `docs/compiling.md` for full build instructions.

## Deploying

```bash
cp deploy-testnet.env.example deploy-testnet.env
# edit deploy-testnet.env with your account details
source deploy-testnet.env && ./deploy-testnet.sh
```

After deploying, verify the on-chain binary matches your local build:

```bash
./verify-build.sh <CONTRACT_ID>
```

See `docs/deploying.md` for step-by-step instructions including manual
deployment and mainnet deployment.

## Function reference

All functions return `Result<T, InvoiceError>` unless noted.

### `create_invoice`

```
create_invoice(
    env: Env,
    issuer: Address,
    payer: Address,
    amount: i128,
    currency: Symbol,
    due_date: u64,
) -> Result<u64, InvoiceError>
```

Creates a new invoice and returns its id. `issuer` must authorize the call.

Errors: `InvalidAmount` (amount ≤ 0), `InvalidDueDate` (due_date ≤ current ledger timestamp).

### `pay_invoice`

```
pay_invoice(
    env: Env,
    invoice_id: u64,
    payer: Address,
    amount: i128,
) -> Result<(), InvoiceError>
```

Records a payment. `payer` must authorize the call. Partial payments are allowed;
status transitions to `PartiallyPaid` or `Paid` accordingly.

Errors: `InvalidAmount`, `InvoiceNotFound`, `UnauthorizedPayer`, `InvoiceNotPayable`,
`PaidAmountOverflow`, `OverpaymentNotAllowed`.

### `get_invoice`

```
get_invoice(env: Env, invoice_id: u64) -> Result<Invoice, InvoiceError>
```

Returns the full `Invoice` struct for the given id. No authorization required.

Errors: `InvoiceNotFound`.

### `cancel_invoice`

```
cancel_invoice(
    env: Env,
    invoice_id: u64,
    issuer: Address,
) -> Result<(), InvoiceError>
```

Cancels an invoice. `issuer` must authorize the call. A fully paid invoice cannot
be cancelled.

Errors: `InvoiceNotFound`, `OnlyIssuerCanCancel`, `CannotCancelPaidInvoice`,
`InvoiceNotPayable` (already cancelled).

### `list_invoices_by_address`

```
list_invoices_by_address(env: Env, addr: Address) -> Vec<u64>
```

Returns all invoice ids associated with an address (whether as issuer or payer).
No authorization required.

### `mark_overdue`

```
mark_overdue(env: Env, invoice_id: u64) -> Result<(), InvoiceError>
```

Permissionless. Transitions a `Created` or `PartiallyPaid` invoice to `Overdue`
if the ledger timestamp is past `due_date`. The caller identity is irrelevant —
the condition is entirely time-driven and consensus-determined.

Errors: `InvoiceNotFound`, `InvoiceNotPayable` (not in a transitionable state, or
due_date not yet passed).

## Security & Access Control

`require_auth()` is called on the relevant address as the first statement in every
function that acts on behalf of a named party:

| Function | Authorized party |
|---|---|
| `create_invoice` | `issuer` |
| `pay_invoice` | `payer` |
| `cancel_invoice` | `issuer` |
| `get_invoice` | none (read-only) |
| `list_invoices_by_address` | none (read-only) |
| `mark_overdue` | none (permissionless by design) |

After the auth check, each function also verifies that the authorized address matches
the role stored on the invoice (`invoice.issuer` or `invoice.payer`). Both checks are
required.

See `docs/access-control.md` for a detailed breakdown.

## Documentation

| Doc | Contents |
|---|---|
| `docs/architecture.md` | Module layout and data flow |
| `docs/access-control.md` | `require_auth` model, per function |
| `docs/data-storage.md` | `DataKey` design, storage tiers, TTL strategy |
| `docs/error-codes.md` | `InvoiceError` variants, which functions raise them, and exact conditions |
| `docs/types.md` | `Invoice` struct and `Status` enum, field by field |
| `docs/testing.md` | How to run tests, what's covered, what isn't |
| `docs/security-considerations.md` | Auth model, overflow guards, audit checklist |
| `docs/integration.md` | Calling this contract from Rust or JS, with code snippets |
| `docs/deploying.md` | Build, deploy, and build-verification steps |
| `docs/compiling.md` | Compiler requirements and build commands |
| `docs/re-entrancy.md` | Soroban's re-entrancy model and applicability here |

## Roadmap

- [x] Invoice lifecycle: create, pay (partial/full), cancel, mark-overdue
- [x] Full auth enforcement via `require_auth()`
- [x] Typed error enum with 9 variants, all tested
- [x] TTL-managed persistent storage (threshold 30 days, extend-to 90 days)
- [x] Events on every state transition (Created, Paid, Cancelled, Overdue)
- [x] Full unit test suite — 41 tests, all error variants covered
- [x] Reproducible build verification script (`verify-build.sh`)
- [x] Automated deploy script (`deploy-testnet.sh`)
- [ ] Testnet deployment (run `deploy-testnet.sh` and add real contract ID here)
- [ ] Token transfer integration (pay_invoice triggers a real asset transfer)

## License

MIT
