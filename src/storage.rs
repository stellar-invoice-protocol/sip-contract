use soroban_sdk::{Address, Env, Symbol, Vec};

use crate::types::Invoice;

const COUNTER: Symbol = symbol_short!("COUNTER");
const PREFIX_INVOICE: Symbol = symbol_short!("INV");
const PREFIX_ADDR_IDX: Symbol = symbol_short!("ADDR_IDX");

pub fn get_counter(env: &Env) -> u64 {
    env.storage().instance().get(&COUNTER).unwrap_or(0)
}

pub fn increment_counter(env: &Env) -> u64 {
    let counter = get_counter(env) + 1;
    env.storage().instance().set(&COUNTER, &counter);
    counter
}

pub fn store_invoice(env: &Env, invoice: &Invoice) {
    let key = (PREFIX_INVOICE, invoice.id);
    env.storage().persistent().set(&key, invoice);
}

pub fn load_invoice(env: &Env, id: u64) -> Option<Invoice> {
    let key = (PREFIX_INVOICE, id);
    env.storage().persistent().get(&key)
}

pub fn push_invoice_to_address(env: &Env, addr: &Address, id: u64) {
    let key = (PREFIX_ADDR_IDX, addr);
    let mut list: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    list.push_back(id);
    env.storage().persistent().set(&key, &list);
}

pub fn get_invoices_for_address(env: &Env, addr: &Address) -> Vec<u64> {
    let key = (PREFIX_ADDR_IDX, addr);
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}