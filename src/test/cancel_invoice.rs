use soroban_sdk::testutils::Address as _;

use crate::errors::InvoiceError;
use crate::types::Status;

use super::common::*;

#[test]
fn cancel_created_invoice_succeeds() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.cancel_invoice(&id, &issuer);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::Cancelled);
}

#[test]
fn cancel_partially_paid_invoice_succeeds() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &200);
    client.cancel_invoice(&id, &issuer);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.status, Status::Cancelled);
}

#[test]
fn cancel_paid_invoice_returns_error() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.pay_invoice(&id, &payer, &1_000);

    let err = client.try_cancel_invoice(&id, &issuer).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::CannotCancelPaidInvoice));
}

#[test]
fn double_cancel_returns_error() {
    let (_, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    client.cancel_invoice(&id, &issuer);

    let err = client.try_cancel_invoice(&id, &issuer).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotPayable));
}

#[test]
fn cancel_by_wrong_issuer_returns_error() {
    let (env, client, issuer, payer) = setup();
    let id = create_default_invoice(&client, &issuer, &payer);

    let impostor = soroban_sdk::Address::generate(&env);

    let err = client.try_cancel_invoice(&id, &impostor).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::OnlyIssuerCanCancel));
}

#[test]
fn cancel_unknown_invoice_returns_not_found() {
    let (_, client, issuer, _) = setup();

    let err = client.try_cancel_invoice(&999, &issuer).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotFound));
}
