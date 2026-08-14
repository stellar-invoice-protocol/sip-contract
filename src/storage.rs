use soroban_sdk::{contracttype, Address, Env, Vec};

use crate::types::Invoice;

// ---------------------------------------------------------------------------
// TTL constants
// Stellar closes a ledger roughly every 5 seconds.
// 17_280 ledgers ≈ 1 day  (17_280 * 5s = 86_400s)
// threshold: ~30 days  →  if TTL drops below this, extend
// extend_to: ~90 days  →  new TTL after extension
// ---------------------------------------------------------------------------
const TTL_THRESHOLD: u32 = 17_280 * 30; //  518_400 ledgers
const TTL_EXTEND_TO: u32 = 17_280 * 90; // 1_555_200 ledgers

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Global invoice counter — stored in instance storage.
    Counter,
    /// One entry per invoice id — stored in persistent storage.
    Invoice(u64),
    /// Address-to-invoice-id index — stored in persistent storage.
    AddressIndex(Address),
}

// ---------------------------------------------------------------------------
// Instance-level helpers
// ---------------------------------------------------------------------------

/// Extend the instance-storage TTL. Call this once at the top of every
/// contract entrypoint that touches storage.
pub fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn get_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Counter)
        .unwrap_or(0)
}

pub fn increment_counter(env: &Env) -> u64 {
    let counter = get_counter(env) + 1;
    env.storage().instance().set(&DataKey::Counter, &counter);
    counter
}

// ---------------------------------------------------------------------------
// Persistent invoice helpers
// ---------------------------------------------------------------------------

pub fn store_invoice(env: &Env, invoice: &Invoice) {
    let key = DataKey::Invoice(invoice.id);
    env.storage().persistent().set(&key, invoice);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn load_invoice(env: &Env, id: u64) -> Option<Invoice> {
    let key = DataKey::Invoice(id);
    let result: Option<Invoice> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    }
    result
}

// ---------------------------------------------------------------------------
// Persistent address-index helpers
// ---------------------------------------------------------------------------

pub fn push_invoice_to_address(env: &Env, addr: &Address, id: u64) {
    let key = DataKey::AddressIndex(addr.clone());
    let mut list: Vec<u64> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));

    list.push_back(id);
    env.storage().persistent().set(&key, &list);
    env.storage()
        .persistent()
        .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

pub fn get_invoices_for_address(env: &Env, addr: &Address) -> Vec<u64> {
    let key = DataKey::AddressIndex(addr.clone());
    let result: Option<Vec<u64>> = env.storage().persistent().get(&key);
    if let Some(ref list) = result {
        // Only bump if the list actually exists.
        let _ = list; // suppress unused-variable warning
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_THRESHOLD, TTL_EXTEND_TO);
    }
    result.unwrap_or_else(|| Vec::new(env))
}
