use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

use crate::{
    errors::InvoiceError,
    events,
    storage::{
        bump_instance, get_invoices_for_address, increment_counter, load_invoice,
        push_invoice_to_address, store_invoice,
    },
    types::{Invoice, Status},
};

#[contract]
pub struct StellarInvoiceContract;

#[contractimpl]
impl StellarInvoiceContract {
    /// Create a new invoice.
    ///
    /// `issuer` must authorize this call — it is the party asserting the debt exists.
    /// Returns the new invoice id.
    ///
    /// Errors
    /// - `InvalidAmount`  — amount is zero or negative.
    /// - `InvalidDueDate` — due_date is not strictly after the current ledger timestamp.
    pub fn create_invoice(
        env: Env,
        issuer: Address,
        payer: Address,
        amount: i128,
        currency: Symbol,
        due_date: u64,
    ) -> Result<u64, InvoiceError> {
        // Auth: the issuer must sign this transaction.
        issuer.require_auth();

        bump_instance(&env);

        if amount <= 0 {
            return Err(InvoiceError::InvalidAmount);
        }
        if due_date <= env.ledger().timestamp() {
            return Err(InvoiceError::InvalidDueDate);
        }

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

        Ok(id)
    }

    /// Record a payment against an invoice.
    ///
    /// `payer` must authorize this call.
    ///
    /// Errors
    /// - `InvalidAmount`        — amount is zero or negative.
    /// - `InvoiceNotFound`      — no invoice with that id exists.
    /// - `UnauthorizedPayer`    — caller is not the invoice's designated payer.
    /// - `InvoiceNotPayable`    — invoice is already Paid or Cancelled.
    /// - `PaidAmountOverflow`   — arithmetic overflow on paid_amount accumulation.
    /// - `OverpaymentNotAllowed`— payment would exceed the invoice amount.
    pub fn pay_invoice(
        env: Env,
        invoice_id: u64,
        payer: Address,
        amount: i128,
    ) -> Result<(), InvoiceError> {
        // Auth: the designated payer must sign this transaction.
        payer.require_auth();

        bump_instance(&env);

        if amount <= 0 {
            return Err(InvoiceError::InvalidAmount);
        }

        let mut invoice =
            load_invoice(&env, invoice_id).ok_or(InvoiceError::InvoiceNotFound)?;

        if invoice.payer != payer {
            return Err(InvoiceError::UnauthorizedPayer);
        }

        if matches!(invoice.status, Status::Cancelled | Status::Paid) {
            return Err(InvoiceError::InvoiceNotPayable);
        }

        let new_paid = invoice
            .paid_amount
            .checked_add(amount)
            .ok_or(InvoiceError::PaidAmountOverflow)?;

        if new_paid > invoice.amount {
            return Err(InvoiceError::OverpaymentNotAllowed);
        }

        invoice.paid_amount = new_paid;
        invoice.status = if new_paid == invoice.amount {
            Status::Paid
        } else {
            Status::PartiallyPaid
        };

        store_invoice(&env, &invoice);

        events::emit_invoice_paid(&env, invoice_id, &payer, amount, new_paid);

        Ok(())
    }

    /// Return the full `Invoice` struct for a given id.
    ///
    /// Errors
    /// - `InvoiceNotFound` — no invoice with that id exists.
    pub fn get_invoice(env: Env, invoice_id: u64) -> Result<Invoice, InvoiceError> {
        bump_instance(&env);
        load_invoice(&env, invoice_id).ok_or(InvoiceError::InvoiceNotFound)
    }

    /// Cancel an invoice.
    ///
    /// Only the issuer may cancel. Cancellation is permitted in Created,
    /// PartiallyPaid, and Overdue states. A fully Paid invoice cannot be
    /// cancelled; that transition is irreversible.
    ///
    /// `issuer` must authorize this call.
    ///
    /// Errors
    /// - `InvoiceNotFound`       — no invoice with that id exists.
    /// - `OnlyIssuerCanCancel`   — caller is not the invoice's issuer.
    /// - `CannotCancelPaidInvoice` — invoice is already fully paid.
    /// - `InvoiceNotPayable`     — invoice is already cancelled.
    pub fn cancel_invoice(
        env: Env,
        invoice_id: u64,
        issuer: Address,
    ) -> Result<(), InvoiceError> {
        // Auth: only the issuer may cancel.
        issuer.require_auth();

        bump_instance(&env);

        let mut invoice =
            load_invoice(&env, invoice_id).ok_or(InvoiceError::InvoiceNotFound)?;

        if invoice.issuer != issuer {
            return Err(InvoiceError::OnlyIssuerCanCancel);
        }

        if matches!(invoice.status, Status::Paid) {
            return Err(InvoiceError::CannotCancelPaidInvoice);
        }

        if matches!(invoice.status, Status::Cancelled) {
            // Already cancelled — idempotent guard surfaces as InvoiceNotPayable
            // (the cancel action is not applicable to the current state).
            return Err(InvoiceError::InvoiceNotPayable);
        }

        invoice.status = Status::Cancelled;
        store_invoice(&env, &invoice);

        events::emit_invoice_cancelled(&env, invoice_id, &issuer);

        Ok(())
    }

    /// Return all invoice ids associated with an address (as issuer or payer).
    pub fn list_invoices_by_address(env: Env, addr: Address) -> Vec<u64> {
        bump_instance(&env);
        get_invoices_for_address(&env, &addr)
    }

    /// Transition an invoice to Overdue if the ledger timestamp is past its
    /// due_date and it has not yet been paid or cancelled.
    ///
    /// This function is intentionally permissionless — anyone may call it
    /// because the state transition is purely time-driven and objective: the
    /// ledger timestamp is consensus-determined and cannot be manipulated by
    /// the caller. Restricting the caller would add friction without adding
    /// security.
    ///
    /// Errors
    /// - `InvoiceNotFound`   — no invoice with that id exists.
    /// - `InvoiceNotPayable` — invoice is already in a terminal state
    ///                         (Paid, Cancelled) or already Overdue, or the
    ///                         due_date has not yet passed.
    pub fn mark_overdue(env: Env, invoice_id: u64) -> Result<(), InvoiceError> {
        bump_instance(&env);

        let mut invoice =
            load_invoice(&env, invoice_id).ok_or(InvoiceError::InvoiceNotFound)?;

        // Only Created or PartiallyPaid invoices past their due_date can
        // transition to Overdue.
        let is_active = matches!(invoice.status, Status::Created | Status::PartiallyPaid);
        let is_past_due = env.ledger().timestamp() > invoice.due_date;

        if !is_active || !is_past_due {
            return Err(InvoiceError::InvoiceNotPayable);
        }

        invoice.status = Status::Overdue;
        store_invoice(&env, &invoice);

        events::emit_invoice_overdue(&env, invoice_id, invoice.due_date);

        Ok(())
    }
}
