/// Shared test helpers — imported by the other test modules.
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

use crate::contract::StellarInvoiceContract;
use crate::contract::StellarInvoiceContractClient;

/// Ledger timestamp we use as "now" in tests.
pub const NOW: u64 = 1_700_000_000;
/// A due_date that is comfortably in the future relative to `NOW`.
pub const FUTURE_DUE: u64 = NOW + 86_400; // +1 day
/// A due_date in the past relative to `NOW`.
pub const PAST_DUE: u64 = NOW - 1;

/// Build a fresh `Env` with the ledger timestamp set to `NOW`, register the
/// contract, and return both together with two pre-generated addresses.
///
/// `mock_all_auths()` is called so business-logic tests don't have to wire up
/// auth entries manually. Authorization correctness is tested separately in
/// `authorization.rs`.
pub fn setup() -> (Env, StellarInvoiceContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    use soroban_sdk::testutils::Ledger as _;
    env.ledger().set_timestamp(NOW);

    let contract_id = env.register(StellarInvoiceContract, ());
    let client = StellarInvoiceContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let payer = Address::generate(&env);

    (env, client, issuer, payer)
}

/// Convenience: create one invoice and return its id.
/// The non-`try_*` client methods return `T` directly (the SDK panics on error),
/// so no `.unwrap()` is needed.
pub fn create_default_invoice(
    client: &StellarInvoiceContractClient,
    issuer: &Address,
    payer: &Address,
) -> u64 {
    client.create_invoice(
        issuer,
        payer,
        &1_000,
        &symbol_short!("XLM"),
        &FUTURE_DUE,
    )
}
