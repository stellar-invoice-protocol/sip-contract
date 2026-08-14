# Makefile — mirrors exactly what CI runs so you can reproduce it locally.
#
# Targets:
#   make fmt       — format all Rust source
#   make lint      — clippy with -D warnings (same flags as CI)
#   make test      — cargo test --all-features
#   make build     — cargo build for wasm32 release target
#   make all       — fmt + lint + test + build in sequence
#   make help      — list targets

WASM_TARGET = wasm32-unknown-unknown
WASM_ARTIFACT = target/$(WASM_TARGET)/release/stellar_invoice_protocol.wasm

.PHONY: fmt lint test build all help

## fmt: format all Rust source with rustfmt
fmt:
	cargo fmt --all

## lint: run clippy with -D warnings (mirrors CI)
lint:
	cargo clippy --all-targets -- -D warnings

## test: run the full unit test suite
test:
	cargo test --all-features

## build: compile the release WASM binary
build:
	cargo build --target $(WASM_TARGET) --release --locked
	@echo ""
	@echo "WASM artifact: $(WASM_ARTIFACT)"

## all: run fmt, lint, test, and build in sequence
all: fmt lint test build

## help: list available targets
help:
	@echo ""
	@echo "Available targets:"
	@grep -E '^## ' Makefile | sed 's/^## /  /'
	@echo ""
