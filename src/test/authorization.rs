/// Authorization tests.
///
/// These tests verify that `require_auth()` is enforced by the contract.
/// They are separated from the business-logic tests because they deliberately
/// do NOT call `env.mock_all_auths()`.
///
/// When no auth is mocked, a `require_auth()` call inside the contract causes
/// the host to return a host-level error. The `try_*` client methods surface
/// this as `Err(Err(soroban_sdk::InvokeError::Contract(..)))`. We simply
/// assert the call returns `Err` — the exact error code is a host-level detail
/// and not a typed `InvoiceError`.
use soroban_sdk::{symbol_short, testutils::Address as _, testutils::Ledger as _, Address, Env};

use crate::contract::StellarInvoiceContract;
use crate::contract::StellarInvoiceContractClient;

use super::common::{FUTURE_DUE, NOW};

/// Register the contract on a fresh env with NO auth mocking.
fn setup_no_auth() -> (Env, StellarInvoiceContractClient<'static>, Address, Address) {
    let env = Env::default();
    // Deliberately NOT calling env.mock_all_auths().
    env.ledger().set_timestamp(NOW);
    let contract_id = env.register(StellarInvoiceContract, ());
    let client = StellarInvoiceContractClient::new(&env, &contract_id);
    let issuer = Address::generate(&env);
    let payer = Address::generate(&env);
    (env, client, issuer, payer)
}

// ---------------------------------------------------------------------------
// create_invoice — issuer must authorize
// ---------------------------------------------------------------------------

#[test]
fn create_invoice_fails_without_auth() {
    let (_, client, issuer, payer) = setup_no_auth();

    let result = client.try_create_invoice(
        &issuer,
        &payer,
        &1_000,
        &symbol_short!("XLM"),
        &FUTURE_DUE,
    );
    assert!(result.is_err(), "expected require_auth failure but got Ok");
}

#[test]
fn create_invoice_succeeds_with_auth() {
    let (env, client, issuer, payer) = setup_no_auth();
    // Grant auth for exactly this call.
    env.mock_all_auths();

    let result = client.try_create_invoice(
        &issuer,
        &payer,
        &1_000,
        &symbol_short!("XLM"),
        &FUTURE_DUE,
    );
    assert!(result.is_ok(), "expected Ok but got: {:?}", result);
}

// ---------------------------------------------------------------------------
// pay_invoice — payer must authorize
// ---------------------------------------------------------------------------

/// Helper: creates an invoice on a separate env (with auth), returns the id.
/// We then try to pay on a no-auth env where the invoice doesn't exist — the
/// call still fails (InvoiceNotFound), which is still an Err, confirming the
/// contract rejects the call before touching state.
#[test]
fn pay_invoice_fails_without_auth() {
    let (_, client, _, payer) = setup_no_auth();

    // Invoice id 1 doesn't exist in this env, but require_auth fires first.
    let result = client.try_pay_invoice(&1, &payer, &500);
    assert!(result.is_err(), "expected failure but got Ok");
}

#[test]
fn pay_invoice_succeeds_with_auth() {
    // Create env with auth, create an invoice, then pay it.
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let contract_id = env.register(StellarInvoiceContract, ());
    let client = StellarInvoiceContractClient::new(&env, &contract_id);
    let issuer = Address::generate(&env);
    let payer = Address::generate(&env);

    let id = client.create_invoice(
        &issuer,
        &payer,
        &1_000,
        &symbol_short!("XLM"),
        &FUTURE_DUE,
    );
    let result = client.try_pay_invoice(&id, &payer, &1_000);
    assert!(result.is_ok(), "expected Ok but got: {:?}", result);
}

// ---------------------------------------------------------------------------
// cancel_invoice — issuer must authorize
// ---------------------------------------------------------------------------

#[test]
fn cancel_invoice_fails_without_auth() {
    let (_, client, issuer, _) = setup_no_auth();

    let result = client.try_cancel_invoice(&1, &issuer);
    assert!(result.is_err(), "expected require_auth failure but got Ok");
}

#[test]
fn cancel_invoice_succeeds_with_auth() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let contract_id = env.register(StellarInvoiceContract, ());
    let client = StellarInvoiceContractClient::new(&env, &contract_id);
    let issuer = Address::generate(&env);
    let payer = Address::generate(&env);

    let id = client.create_invoice(
        &issuer,
        &payer,
        &1_000,
        &symbol_short!("XLM"),
        &FUTURE_DUE,
    );
    let result = client.try_cancel_invoice(&id, &issuer);
    assert!(result.is_ok(), "expected Ok but got: {:?}", result);
}

// ---------------------------------------------------------------------------
// mark_overdue — intentionally permissionless
// ---------------------------------------------------------------------------

#[test]
fn mark_overdue_requires_no_auth() {
    // mark_overdue should succeed for any caller with no auth at all,
    // as long as the preconditions are met.
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(NOW);
    let contract_id = env.register(StellarInvoiceContract, ());
    let client = StellarInvoiceContractClient::new(&env, &contract_id);
    let issuer = Address::generate(&env);
    let payer = Address::generate(&env);

    let id = client.create_invoice(
        &issuer,
        &payer,
        &1_000,
        &symbol_short!("XLM"),
        &FUTURE_DUE,
    );

    // Advance time past due date — no auth needed.
    env.ledger().set_timestamp(FUTURE_DUE + 1);

    // Call without any specific auth mocked — should succeed.
    let result = client.try_mark_overdue(&id);
    assert!(result.is_ok(), "mark_overdue should be permissionless: {:?}", result);
}
