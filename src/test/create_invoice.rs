use soroban_sdk::{symbol_short, testutils::Address as _};

use crate::errors::InvoiceError;
use crate::types::Status;

use super::common::*;

#[test]
fn create_invoice_returns_sequential_ids() {
    let (_, client, issuer, payer) = setup();

    let id1 = client.create_invoice(&issuer, &payer, &500, &symbol_short!("XLM"), &FUTURE_DUE);
    let id2 = client.create_invoice(&issuer, &payer, &500, &symbol_short!("XLM"), &FUTURE_DUE);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn create_invoice_stores_correct_fields() {
    let (_, client, issuer, payer) = setup();

    let id = client.create_invoice(&issuer, &payer, &1_000, &symbol_short!("XLM"), &FUTURE_DUE);

    let inv = client.get_invoice(&id);
    assert_eq!(inv.id, id);
    assert_eq!(inv.issuer, issuer);
    assert_eq!(inv.payer, payer);
    assert_eq!(inv.amount, 1_000);
    assert_eq!(inv.due_date, FUTURE_DUE);
    assert_eq!(inv.status, Status::Created);
    assert_eq!(inv.paid_amount, 0);
}

#[test]
fn create_invoice_indexes_both_addresses() {
    let (_, client, issuer, payer) = setup();

    let id = create_default_invoice(&client, &issuer, &payer);

    let by_issuer = client.list_invoices_by_address(&issuer);
    let by_payer = client.list_invoices_by_address(&payer);

    assert!(by_issuer.contains(&id));
    assert!(by_payer.contains(&id));
}

#[test]
fn create_invoice_rejects_zero_amount() {
    let (_, client, issuer, payer) = setup();

    let err = client
        .try_create_invoice(&issuer, &payer, &0, &symbol_short!("XLM"), &FUTURE_DUE)
        .unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvalidAmount));
}

#[test]
fn create_invoice_rejects_negative_amount() {
    let (_, client, issuer, payer) = setup();

    let err = client
        .try_create_invoice(&issuer, &payer, &-1, &symbol_short!("XLM"), &FUTURE_DUE)
        .unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvalidAmount));
}

#[test]
fn create_invoice_rejects_due_date_in_past() {
    let (_, client, issuer, payer) = setup();

    let err = client
        .try_create_invoice(&issuer, &payer, &500, &symbol_short!("XLM"), &PAST_DUE)
        .unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvalidDueDate));
}

#[test]
fn create_invoice_rejects_due_date_equal_to_now() {
    let (_, client, issuer, payer) = setup();

    // due_date == NOW is not strictly in the future.
    let err = client
        .try_create_invoice(&issuer, &payer, &500, &symbol_short!("XLM"), &NOW)
        .unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvalidDueDate));
}

#[test]
fn get_invoice_unknown_id_returns_not_found() {
    let (_, client, _, _) = setup();

    let err = client.try_get_invoice(&999).unwrap_err();
    assert_eq!(err, Ok(InvoiceError::InvoiceNotFound));
}

#[test]
fn list_invoices_unknown_address_returns_empty() {
    let (env, client, _, _) = setup();
    let stranger = soroban_sdk::Address::generate(&env);
    let list = client.list_invoices_by_address(&stranger);
    assert_eq!(list.len(), 0);
}
