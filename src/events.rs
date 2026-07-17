use soroban_sdk::{symbol_short, Env, Symbol};

pub fn emit_invoice_created(
    env: &Env,
    id: u64,
    issuer: &soroban_sdk::Address,
    payer: &soroban_sdk::Address,
    amount: i128,
    currency: Symbol,
    due_date: u64,
) {
    env.events().publish(
        (symbol_short!("Invoice"), symbol_short!("Created")),
        (id, issuer, payer, amount, currency, due_date),
    );
}

// Future implementation as required