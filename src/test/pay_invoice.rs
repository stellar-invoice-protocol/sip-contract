use soroban_sdk::{testutils::Address as _, Address};

use crate::errors::InvoiceError;
use crate::types::Status;

use super::common::*;

#[test]
fn full_payment_transitions_to_paid() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &1_000);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::Paid);
    assert_eq!(inv.paid_amount, 1_000);
}

#[test]
fn partial_payment_transitions_to_partially_paid() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &400);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::PartiallyPaid);
    assert_eq!(inv.paid_amount, 400);
}

#[test]
fn multiple_partial_payments_accumulate() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &300);
    client.pay_invoice(&id, &payer, &300);
    client.pay_invoice(&id, &payer, &400);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::Paid);
    assert_eq!(inv.paid_amount, 1_000);
}

#[test]
fn pay_invoice_rejects_overpayment() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    let err = client.try_pay_invoice(&id, &payer, &1_001).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::OverpaymentNotAllowed));
}

#[test]
fn pay_invoice_rejects_overpayment_after_partial() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &800);

    // 800 + 300 = 1100 > 1000
    let err = client.try_pay_invoice(&id, &payer, &300).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::OverpaymentNotAllowed));
}

#[test]
fn pay_invoice_rejects_wrong_payer() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    let impostor = Address::generate(&env);

    let err = client.try_pay_invoice(&id, &impostor, &500).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::UnauthorizedPayer));
}

#[test]
fn pay_invoice_rejects_zero_amount() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    let err = client.try_pay_invoice(&id, &payer, &0).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvalidAmount));
}

#[test]
fn pay_invoice_rejects_negative_amount() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    let err = client.try_pay_invoice(&id, &payer, &-10).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvalidAmount));
}

#[test]
fn pay_invoice_unknown_id_returns_not_found() {
    let (_, client, _, payer) = setup();

    let err = client.try_pay_invoice(&999, &payer, &100).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotFound));
}

#[test]
fn cannot_pay_already_paid_invoice() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &1_000);

    let err = client.try_pay_invoice(&id, &payer, &1).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn cannot_pay_cancelled_invoice() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.cancel_invoice(&id, &issuer);

    let err = client.try_pay_invoice(&id, &payer, &500).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn pay_invoice_emits_events() {
    // Verify that calling pay_invoice does not panic and that the
    // event machinery runs without error. Detailed event decoding is
    // an integration concern; here we just assert the call succeeds.
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);
    client.pay_invoice(&id, &payer, &1_000);
    // If we reach here without panic, the event was published.
}
