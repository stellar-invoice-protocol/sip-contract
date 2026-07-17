use soroban_sdk::{contractimpl, Address, Env, Symbol};

use crate::{
    errors::*,
    events,
    storage::*,
    types::{Invoice, Status},
};

pub struct StellarInvoiceContract;

#[contractimpl]
impl StellarInvoiceContract {
    pub fn create_invoice(
        env: Env,
        issuer: Address,
        payer: Address,
        amount: i128,
        currency: Symbol,
        due_date: u64,
    ) -> u64 {
        let id = increment_counter(&env);
        let created_at = env.ledger().timestamp();

        let invoice = Invoice {
            id,
            issuer: issuer.clone(),
            payer: payer.clone(),
            amount,
            currency: currency.clone(),
            due_date,
            status: Status::Created,
            created_at,
            paid_amount: 0,
        };

        store_invoice(&env, &invoice);
        push_invoice_to_address(&env, &issuer, id);
        push_invoice_to_address(&env, &payer, id);

        events::emit_invoice_created(&env, id, &issuer, &payer, amount, currency, due_date);

        id
    }

    pub fn pay_invoice(env: Env, invoice_id: u64, payer: Address, amount: i128) {
        let mut invoice = load_invoice(&env, invoice_id).expect(ERR_INVOICE_NOT_FOUND);

        if invoice.payer != payer {
            panic_with_error!(env, ERR_UNAUTHORIZED_PAYER);
        }
        if matches!(invoice.status, Status::Cancelled | Status::Paid) {
            panic_with_error!(env, ERR_INVOICE_NOT_PAYABLE);
        }

        let new_paid = invoice
            .paid_amount
            .checked_add(amount)
            .expect("overflow");

        if new_paid > invoice.amount {
            panic_with_error!(env, ERR_OVERPAYMENT);
        }

        invoice.paid_amount = new_paid;
        invoice.status = match new_paid {
            0 => Status::Created,
            x if x < invoice.amount => Status::PartiallyPaid,
            _ => Status::Paid,
        };

        store_invoice(&env, &invoice);

        // emit paid event...
    }

    // ... other functions (get_invoice, cancel_invoice, list_invoices_by_address, mark_overdue)
}