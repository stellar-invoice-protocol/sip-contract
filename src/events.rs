use soroban_sdk::{symbol_short, Address, Env, Symbol};

/// Emitted when a new invoice is created.
///
/// Topics : ("Invoice", "Created")
/// Data   : (id, issuer, payer, amount, currency, due_date)
pub fn emit_invoice_created(
    env: &Env,
    id: u64,
    issuer: &Address,
    payer: &Address,
    amount: i128,
    currency: Symbol,
    due_date: u64,
) {
    env.events().publish(
        (symbol_short!("Invoice"), symbol_short!("Created")),
        (id, issuer, payer, amount, currency, due_date),
    );
}

/// Emitted when a payment is recorded against an invoice.
///
/// Topics : ("Invoice", "Paid")
/// Data   : (id, payer, amount_paid, paid_amount_total)
pub fn emit_invoice_paid(env: &Env, id: u64, payer: &Address, amount_paid: i128, paid_total: i128) {
    env.events().publish(
        (symbol_short!("Invoice"), symbol_short!("Paid")),
        (id, payer, amount_paid, paid_total),
    );
}

/// Emitted when an invoice is cancelled by the issuer.
///
/// Topics : ("Invoice", "Cancelled")
/// Data   : (id, issuer)
pub fn emit_invoice_cancelled(env: &Env, id: u64, issuer: &Address) {
    env.events().publish(
        (symbol_short!("Invoice"), symbol_short!("Cancelled")),
        (id, issuer),
    );
}

/// Emitted when an invoice is transitioned to Overdue status.
///
/// Topics : ("Invoice", "Overdue")
/// Data   : (id, due_date)
pub fn emit_invoice_overdue(env: &Env, id: u64, due_date: u64) {
    env.events().publish(
        (symbol_short!("Invoice"), symbol_short!("Overdue")),
        (id, due_date),
    );
}
