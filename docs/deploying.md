# Deploying

## Prerequisites

- Rust with the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- The Stellar CLI (`stellar`):
  ```bash
  cargo install --locked stellar-cli --features opt
  ```
- A funded Stellar account on the network you are deploying to. For testnet,
  use Friendbot: `stellar keys fund --network testnet`

## 1. Build the WASM

```bash
cargo build --target wasm32-unknown-unknown --release
```

The artifact is at:
```
target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm
```

Alternatively, use the Stellar CLI which runs the build and optimises the binary:
```bash
stellar contract build
```

## 2. Deploy to testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm \
  --source <YOUR_ACCOUNT_ALIAS> \
  --network testnet
```

This prints the contract id (a `C…` address). Save it — you will need it for all
subsequent invocations.

## 3. Invoke a function (example)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ISSUER_ACCOUNT_ALIAS> \
  --network testnet \
  -- create_invoice \
  --issuer <ISSUER_ADDRESS> \
  --payer <PAYER_ADDRESS> \
  --amount 1000 \
  --currency XLM \
  --due_date 1800000000
```

## 4. Deploy to mainnet

Replace `--network testnet` with `--network mainnet` throughout. Ensure your
source account is funded on mainnet before deploying.

## Notes

- The `[profile.release]` block in `Cargo.toml` enables `lto = true` and
  `opt-level = "z"`, which minimises WASM binary size. These settings are applied
  automatically by `cargo build --release`.
- There is no initialisation function — the contract is ready to use immediately
  after deployment. The invoice counter starts at 0 (first invoice will be id 1).
