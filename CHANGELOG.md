# Changelog

All notable changes to the **Stellar Invoice Protocol** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Modular codebase structure (`types.rs`, `storage.rs`, `errors.rs`, `events.rs`, `contract.rs`)
- Improved storage keys using tuples instead of string concatenation
- Custom error handling with `panic_with_error!`
- Separate events module
- Better separation of concerns for maintainability

### Changed
- Refactored from monolithic `lib.rs` into proper Rust modules
- Storage helpers moved to `storage.rs`
- Updated key generation for address indexing (more reliable and efficient)

### Fixed
- Removed unsafe `format!("{:?}", addr)` usage in storage keys

---

## [0.1.0] - 2026-07-17

### Added
- Core invoice lifecycle: `create_invoice`, `pay_invoice` (full + partial), `get_invoice`
- Invoice status management: `Created`, `PartiallyPaid`, `Paid`, `Overdue`, `Cancelled`
- Cancel invoice functionality (`cancel_invoice`)
- List invoices by address (`list_invoices_by_address`)
- Mark invoice as overdue (`mark_overdue`)
- Event emission for key actions (Created, Paid, Cancelled, Overdue)
- Unit tests covering full payment, partial payment, overdue, authorization, and overpayment guards
- Basic README and CONTRIBUTING documentation

### Technical
- Built with Soroban SDK `0.22.0`
- Uses persistent storage for invoices and address indices
- Instance storage for global counter

---

## [0.0.1] - 2026-07-10 (Initial Scaffold)

- Initial single-file prototype
