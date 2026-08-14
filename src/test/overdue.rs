// `Ledger::set_timestamp` is a testutils extension method.
use soroban_sdk::testutils::Ledger as _;

use crate::errors::InvoiceError;
use crate::types::Status;

use super::common::*;

/// Advance the ledger timestamp one second past the invoice due_date.
fn advance_past_due(env: &soroban_sdk::Env) {
    env.ledger().set_timestamp(FUTURE_DUE + 1);
}

#[test]
fn mark_overdue_transitions_created_invoice() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    advance_past_due(&env);
    client.mark_overdue(&id);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::Overdue);
}

#[test]
fn mark_overdue_transitions_partially_paid_invoice() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &300);

    advance_past_due(&env);
    client.mark_overdue(&id);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::Overdue);
}

#[test]
fn mark_overdue_before_due_date_returns_error() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);
    // Ledger is still at NOW < FUTURE_DUE.

    let err = client.try_mark_overdue(&id).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn mark_overdue_on_paid_invoice_returns_error() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &1_000);

    advance_past_due(&env);
    let err = client.try_mark_overdue(&id).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn mark_overdue_on_cancelled_invoice_returns_error() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.cancel_invoice(&id, &issuer);

    advance_past_due(&env);
    let err = client.try_mark_overdue(&id).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn mark_overdue_already_overdue_returns_error() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    advance_past_due(&env);
    client.mark_overdue(&id);

    // Calling again must fail — Overdue is not Created or PartiallyPaid.
    let err = client.try_mark_overdue(&id).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn mark_overdue_unknown_invoice_returns_not_found() {
    let (_, client, _, _) = setup();

    let err = client.try_mark_overdue(&999).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotFound));
}
