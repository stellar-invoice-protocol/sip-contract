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
cargo build --target wasm32-unknown-unknown --release --locked
```

The artifact is at:
```
target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm
```

Alternatively, use the Stellar CLI which runs the build and applies additional
size optimisations:
```bash
stellar contract build
```

## 2. Deploy to testnet

**Automated (recommended):**

```bash
cp deploy-testnet.env.example deploy-testnet.env
# edit deploy-testnet.env — set STELLAR_SOURCE to your account identity or secret key
source deploy-testnet.env && ./deploy-testnet.sh
```

The script builds the contract, deploys it, and writes the resulting contract
ID to `.last-deploy-testnet`.

**Manual:**

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm \
  --source <YOUR_ACCOUNT_ALIAS> \
  --network testnet
```

This prints the contract ID (a `C…` address). Save it — you will need it for all
subsequent invocations.

## 3. Verify the build

After deploying, confirm that what is on-chain matches your local build:

```bash
./verify-build.sh <CONTRACT_ID>
```

The script:
1. Runs `cargo clean` to remove any prior artifacts.
2. Rebuilds with `--locked` for a deterministic output.
3. Computes the SHA-256 of the local `.wasm` file.
4. Fetches the on-chain WASM via `stellar contract fetch`.
5. Compares the two hashes and prints `MATCH` or `MISMATCH`.

To print the local hash without an on-chain comparison:
```bash
./verify-build.sh
```

The `NETWORK` environment variable controls which network is queried (default:
`testnet`):
```bash
NETWORK=mainnet ./verify-build.sh <CONTRACT_ID>
```

## 4. Invoke a function (example)

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

## 5. Deploy to mainnet

Replace `--network testnet` with `--network mainnet` throughout. Ensure your
source account is funded on mainnet before deploying.

## Notes

- The `[profile.release]` block in `Cargo.toml` sets `lto = true` and
  `opt-level = "z"`, minimising WASM binary size. These settings apply
  automatically on `cargo build --release`.
- There is no initialisation function — the contract is ready immediately after
  deployment. The invoice counter starts at 0; the first invoice will have id 1.
- Never commit `deploy-testnet.env` — it contains your secret key. It is
  already listed in `.gitignore`.
