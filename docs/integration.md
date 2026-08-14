# Integration

This document shows how another contract or a frontend application would call
the Stellar Invoice Protocol contract.

## From a Rust Soroban contract

Add the crate as a dependency (pointing at the published contract id) and use
the generated client, or invoke via `env.invoke_contract`.

```rust
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env};

// Import the types you need to decode return values.
use stellar_invoice_protocol::{Invoice, InvoiceError};

#[contract]
pub struct ExampleCaller;

#[contractimpl]
impl ExampleCaller {
    /// Create an invoice in the SIP contract and record the id.
    pub fn raise_invoice(
        env: Env,
        sip_contract_id: Address,
        issuer: Address,
        payer: Address,
        amount: i128,
        due_date: u64,
    ) -> Result<u64, InvoiceError> {
        let client =
            stellar_invoice_protocol::contract::StellarInvoiceContractClient::new(
                &env,
                &sip_contract_id,
            );
        client.create_invoice(
            &issuer,
            &payer,
            &amount,
            &symbol_short!("XLM"),
            &due_date,
        )
    }
}
```

## From a JavaScript/TypeScript frontend

Use the `@stellar/stellar-sdk` together with a generated TypeScript binding
(produced by `stellar contract bindings typescript`).

```typescript
import { Client } from "./generated/stellar-invoice-protocol";
import { SorobanRpc } from "@stellar/stellar-sdk";

const rpc = new SorobanRpc.Server("https://soroban-testnet.stellar.org");

const client = new Client({
  contractId: "CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX",
  rpcUrl: "https://soroban-testnet.stellar.org",
  networkPassphrase: "Test SDF Network ; September 2015",
});

// Create an invoice (issuer must sign the transaction).
const { result } = await client.create_invoice({
  issuer: issuerKeypair.publicKey(),
  payer: payerKeypair.publicKey(),
  amount: BigInt(1000),
  currency: "XLM",
  due_date: BigInt(Math.floor(Date.now() / 1000) + 86400),
});
console.log("New invoice id:", result);

// Pay the invoice (payer must sign).
await client.pay_invoice({
  invoice_id: result,
  payer: payerKeypair.publicKey(),
  amount: BigInt(1000),
});
```

## Function signatures (quick reference)

```
create_invoice(issuer, payer, amount, currency, due_date) -> Result<u64, InvoiceError>
pay_invoice(invoice_id, payer, amount)                    -> Result<(), InvoiceError>
get_invoice(invoice_id)                                   -> Result<Invoice, InvoiceError>
cancel_invoice(invoice_id, issuer)                        -> Result<(), InvoiceError>
list_invoices_by_address(addr)                            -> Vec<u64>
mark_overdue(invoice_id)                                  -> Result<(), InvoiceError>
```

## Important: no token transfer

This contract records invoice state. It does **not** move tokens. A complete
payment workflow requires a separate token transfer (e.g. via the Stellar
Asset Contract or a custom token) coordinated off-chain or in a wrapper contract.
