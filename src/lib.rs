#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

const MAX_REFERENCE_LENGTH: u32 = 64;

// Invoice status
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Created,
    PartiallyPaid,
    Paid,
    Overdue,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Invoice {
    pub id: u64,
    pub issuer: Address,
    pub payer: Address,
    pub amount: i128,
    pub currency: Symbol,
    pub due_date: u64,
    pub status: Status,
    pub created_at: u64,
    pub paid_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataKey {
    Counter,
    Invoice(u64),
    AddrIdx(Address),
    InvoiceReference(u64),
}

#[contract]
pub struct StellarInvoiceContract;

impl StellarInvoiceContract {
    fn validate_reference(reference: &String) {
        if reference.len() == 0 {
            panic!("empty_invoice_reference");
        }
        if reference.len() > MAX_REFERENCE_LENGTH {
            panic!("invoice_reference_too_long");
        }
    }

    fn get_counter(env: &Env) -> u64 {
        env.storage().instance().get(&DataKey::Counter).unwrap_or(0_u64)
    }

    fn set_counter(env: &Env, v: u64) {
        env.storage().instance().set(&DataKey::Counter, &v);
    }

    fn next_invoice_id(env: &Env) -> u64 {
        let id = Self::get_counter(env) + 1;
        Self::set_counter(env, id);
        id
    }

    fn new_invoice(
        env: &Env,
        id: u64,
        issuer: Address,
        payer: Address,
        amount: i128,
        currency: Symbol,
        due_date: u64,
    ) -> Invoice {
        Invoice {
            id,
            issuer,
            payer,
            amount,
            currency,
            due_date,
            status: Status::Created,
            created_at: env.ledger().timestamp(),
            paid_amount: 0,
        }
    }

    fn store_invoice(env: &Env, invoice: &Invoice) {
        env.storage().persistent().set(&DataKey::Invoice(invoice.id), invoice);
    }

    fn load_invoice(env: &Env, id: u64) -> Option<Invoice> {
        env.storage().persistent().get(&DataKey::Invoice(id))
    }

    fn store_invoice_reference(env: &Env, id: u64, reference: &String) {
        env.storage().persistent().set(&DataKey::InvoiceReference(id), reference);
    }

    fn load_invoice_reference(env: &Env, id: u64) -> Option<String> {
        env.storage().persistent().get(&DataKey::InvoiceReference(id))
    }

    fn has_invoice_reference(env: &Env, id: u64) -> bool {
        env.storage().persistent().has(&DataKey::InvoiceReference(id))
    }

    fn push_invoice_to_address(env: &Env, addr: &Address, id: u64) {
        let key = DataKey::AddrIdx(addr.clone());
        let mut list: Vec<u64> = env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env));
        list.push_back(id);
        env.storage().persistent().set(&key, &list);
    }

    fn get_invoices_for_address(env: &Env, addr: &Address) -> Vec<u64> {
        let key = DataKey::AddrIdx(addr.clone());
        env.storage().persistent().get(&key).unwrap_or_else(|| Vec::new(env))
    }

    fn publish_invoice_created(env: &Env, invoice: &Invoice) {
        env.events().publish(
            (symbol_short!("Invoice"), symbol_short!("Created")),
            (
                invoice.id,
                invoice.issuer.clone(),
                invoice.payer.clone(),
                invoice.amount,
                invoice.currency.clone(),
                invoice.due_date,
            ),
        );
    }

    fn publish_invoice_reference_set(env: &Env, id: u64, reference: &String) {
        env.events().publish((symbol_short!("Invoice"), symbol_short!("RefSet")), (id, reference.clone()));
    }

    fn attach_invoice_reference(env: &Env, id: u64, issuer: &Address, reference: &String) {
        let invoice = Self::load_invoice(env, id).expect("invoice_not_found");
        if invoice.issuer != *issuer {
            panic!("only_issuer_can_set_reference");
        }
        issuer.require_auth();
        Self::validate_reference(reference);
        if Self::has_invoice_reference(env, id) {
            panic!("invoice_reference_already_set");
        }
        Self::store_invoice_reference(env, id, reference);
        Self::publish_invoice_reference_set(env, id, reference);
    }

    fn create_invoice_record(
        env: &Env,
        issuer: Address,
        payer: Address,
        amount: i128,
        currency: Symbol,
        due_date: u64,
    ) -> u64 {
        let id = Self::next_invoice_id(env);
        let invoice = Self::new_invoice(env, id, issuer, payer, amount, currency, due_date);

        Self::store_invoice(env, &invoice);
        Self::push_invoice_to_address(env, &invoice.issuer, id);
        Self::push_invoice_to_address(env, &invoice.payer, id);
        Self::publish_invoice_created(env, &invoice);
        id
    }
}

#[contractimpl]
impl StellarInvoiceContract {
    // Create invoice and return id
    pub fn create_invoice(
        env: Env,
        issuer: Address,
        payer: Address,
        amount: i128,
        currency: Symbol,
        due_date: u64,
    ) -> u64 {
        Self::create_invoice_record(&env, issuer, payer, amount, currency, due_date)
    }

    pub fn create_invoice_with_reference(
        env: Env,
        issuer: Address,
        payer: Address,
        amount: i128,
        currency: Symbol,
        due_date: u64,
        reference: String,
    ) -> u64 {
        let id = Self::create_invoice_record(&env, issuer.clone(), payer, amount, currency, due_date);
        Self::attach_invoice_reference(&env, id, &issuer, &reference);
        id
    }

    pub fn set_invoice_reference(env: Env, invoice_id: u64, issuer: Address, reference: String) {
        Self::attach_invoice_reference(&env, invoice_id, &issuer, &reference);
    }

    pub fn get_invoice_reference(env: Env, invoice_id: u64) -> Option<String> {
        Self::load_invoice(&env, invoice_id).expect("invoice_not_found");
        Self::load_invoice_reference(&env, invoice_id)
    }

    pub fn pay_invoice(env: Env, invoice_id: u64, payer: Address, amount: i128) {
        let mut invoice = StellarInvoiceContract::load_invoice(&env, invoice_id).expect("invoice_not_found");

        // only the payer can pay
        if invoice.payer != payer {
            panic!("unauthorized_payer");
        }
        if invoice.status == Status::Cancelled || invoice.status == Status::Paid {
            panic!("invoice_not_payable");
        }

        // guard against double overpayments beyond i128 range (simple guard)
        let new_paid = invoice.paid_amount.checked_add(amount).expect("overflow_on_paid_amount");
        if new_paid > invoice.amount {
            panic!("overpayment_not_allowed");
        }
        invoice.paid_amount = new_paid;

        // transition status
        if invoice.paid_amount == 0 {
            invoice.status = Status::Created;
        } else if invoice.paid_amount < invoice.amount {
            invoice.status = Status::PartiallyPaid;
        } else {
            invoice.status = Status::Paid;
        }

        StellarInvoiceContract::store_invoice(&env, &invoice);

        env.events().publish(
            (symbol_short!("Invoice"), symbol_short!("Paid")),
            (invoice.id, payer, amount, invoice.paid_amount, invoice.status),
        );
    }

    pub fn get_invoice(env: Env, invoice_id: u64) -> Invoice {
        StellarInvoiceContract::load_invoice(&env, invoice_id).expect("invoice_not_found")
    }

    pub fn cancel_invoice(env: Env, invoice_id: u64, issuer: Address) {
        let mut invoice = StellarInvoiceContract::load_invoice(&env, invoice_id).expect("invoice_not_found");
        if invoice.issuer != issuer {
            panic!("only_issuer_can_cancel");
        }
        if invoice.paid_amount != 0 || invoice.status == Status::Paid {
            panic!("cannot_cancel_paid_invoice");
        }
        invoice.status = Status::Cancelled;
        StellarInvoiceContract::store_invoice(&env, &invoice);
        env.events().publish((symbol_short!("Invoice"), symbol_short!("Cancelled")), (invoice.id, issuer));
    }

    pub fn list_invoices_by_address(env: Env, addr: Address) -> Vec<u64> {
        StellarInvoiceContract::get_invoices_for_address(&env, &addr)
    }

    pub fn mark_overdue(env: Env, invoice_id: u64) {
        let mut invoice = StellarInvoiceContract::load_invoice(&env, invoice_id).expect("invoice_not_found");
        let ts = env.ledger().timestamp();
        if invoice.status != Status::Paid && invoice.status != Status::Cancelled && ts > invoice.due_date {
            invoice.status = Status::Overdue;
            StellarInvoiceContract::store_invoice(&env, &invoice);
            env.events()
                .publish((symbol_short!("Invoice"), symbol_short!("Overdue")), (invoice.id, invoice.due_date, ts));
        }
    }
}

// Unit tests using soroban-sdk testutils
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Env as TestEnv, Address, BytesN, Env, Symbol};

    fn addr_from_byte(env: &Env, b: u8) -> Address {
        Address::from_contract_id(&env, &BytesN::from_array(&env, &[b; 32]))
    }

    #[test]
    fn test_full_payment() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 1);
        let payer = addr_from_byte(&env, 2);
        let currency = Symbol::short("XLM");

        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            1000_i128,
            currency.clone(),
            env.ledger().timestamp() + 1000,
        );
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 1000_i128);
        let inv = StellarInvoiceContract::get_invoice(env.clone(), id);
        assert_eq!(inv.paid_amount, 1000_i128);
        assert_eq!(inv.status, Status::Paid);
    }

    #[test]
    fn test_partial_payment() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 3);
        let payer = addr_from_byte(&env, 4);
        let currency = Symbol::short("XLM");

        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            1000_i128,
            currency.clone(),
            env.ledger().timestamp() + 1000,
        );
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 400_i128);
        let inv = StellarInvoiceContract::get_invoice(env.clone(), id);
        assert_eq!(inv.paid_amount, 400_i128);
        assert_eq!(inv.status, Status::PartiallyPaid);
    }

    #[test]
    fn test_invoice_ids_are_sequential() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 12);
        let payer = addr_from_byte(&env, 13);
        let due_date = env.ledger().timestamp() + 1000;

        let first = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            100_i128,
            Symbol::short("XLM"),
            due_date,
        );
        let second = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer,
            payer,
            200_i128,
            Symbol::short("XLM"),
            due_date,
        );

        assert_eq!(first, 1);
        assert_eq!(second, 2);
    }

    #[test]
    fn test_created_invoice_is_indexed_for_both_parties() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 14);
        let payer = addr_from_byte(&env, 15);
        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            100_i128,
            Symbol::short("XLM"),
            env.ledger().timestamp() + 1000,
        );

        assert_eq!(StellarInvoiceContract::list_invoices_by_address(env.clone(), issuer).get(0), Some(id),);
        assert_eq!(StellarInvoiceContract::list_invoices_by_address(env, payer).get(0), Some(id),);
    }

    #[test]
    fn test_create_invoice_with_reference() {
        let env = TestEnv::default();
        env.mock_all_auths();
        let issuer = addr_from_byte(&env, 16);
        let reference = String::from_str(&env, "PO-2026-0041");

        let id = StellarInvoiceContract::create_invoice_with_reference(
            env.clone(),
            issuer,
            addr_from_byte(&env, 17),
            100_i128,
            Symbol::short("XLM"),
            env.ledger().timestamp() + 1000,
            reference.clone(),
        );

        assert_eq!(StellarInvoiceContract::get_invoice_reference(env, id), Some(reference));
    }

    #[test]
    fn test_plain_invoice_has_no_reference() {
        let env = TestEnv::default();
        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            addr_from_byte(&env, 18),
            addr_from_byte(&env, 19),
            100_i128,
            Symbol::short("XLM"),
            env.ledger().timestamp() + 1000,
        );

        assert_eq!(StellarInvoiceContract::get_invoice_reference(env, id), None);
    }

    #[test]
    fn test_attach_reference_to_existing_invoice() {
        let env = TestEnv::default();
        env.mock_all_auths();
        let issuer = addr_from_byte(&env, 20);
        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            addr_from_byte(&env, 21),
            100_i128,
            Symbol::short("XLM"),
            env.ledger().timestamp() + 1000,
        );
        let reference = String::from_str(&env, "CUSTOMER-7");

        StellarInvoiceContract::set_invoice_reference(env.clone(), id, issuer, reference.clone());

        assert_eq!(StellarInvoiceContract::get_invoice_reference(env, id), Some(reference));
    }

    #[test]
    fn test_overdue_transition() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 5);
        let payer = addr_from_byte(&env, 6);
        let currency = Symbol::short("XLM");

        // create invoice due immediately
        let now = env.ledger().timestamp();
        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            1000_i128,
            currency.clone(),
            now,
        );

        // fast-forward ledger timestamp in testutils
        env.ledger().set(now + 100);

        StellarInvoiceContract::mark_overdue(env.clone(), id);
        let inv = StellarInvoiceContract::get_invoice(env.clone(), id);
        assert_eq!(inv.status, Status::Overdue);
    }

    #[test]
    #[should_panic]
    fn test_unauthorized_cancel() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 7);
        let payer = addr_from_byte(&env, 8);
        let attacker = addr_from_byte(&env, 9);
        let currency = Symbol::short("XLM");

        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            1000_i128,
            currency.clone(),
            env.ledger().timestamp() + 1000,
        );
        // attacker tries to cancel
        StellarInvoiceContract::cancel_invoice(env.clone(), id, attacker.clone());
    }

    #[test]
    #[should_panic]
    fn test_double_payment_guard() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 10);
        let payer = addr_from_byte(&env, 11);
        let currency = Symbol::short("XLM");

        let id = StellarInvoiceContract::create_invoice(
            env.clone(),
            issuer.clone(),
            payer.clone(),
            1000_i128,
            currency.clone(),
            env.ledger().timestamp() + 1000,
        );
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 1000_i128);
        // second payment should panic due to overpayment
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 1_i128);
    }
}
