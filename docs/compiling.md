# Compiling

## Requirements

| Tool | Minimum version | Install |
|---|---|---|
| Rust | stable (1.74+) | `rustup update stable` |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |
| stellar-cli | latest | `cargo install --locked stellar-cli --features opt` |

## Run unit tests (native, no WASM)

```bash
cargo test
```

The `rlib` entry in `crate-type` (alongside `cdylib`) is what makes this work —
`cargo test` needs to link a native library, not a WASM blob.

## Build the WASM binary (release)

```bash
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/stellar_invoice_protocol.wasm`

The release profile (`Cargo.toml`) applies:
- `opt-level = "z"` — optimise for binary size
- `lto = true` — link-time optimisation
- `strip = "symbols"` — remove debug symbols
- `overflow-checks = true` — keep arithmetic safety in release
- `panic = "abort"` — no unwinding in WASM

## Build with the Stellar CLI (equivalent, with additional size optimisation)

```bash
stellar contract build
```

## Run Clippy

```bash
cargo clippy --all-targets -- -D warnings
```

## Check formatting

```bash
cargo fmt --check
```
