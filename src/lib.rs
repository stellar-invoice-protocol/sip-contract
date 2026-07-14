#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

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
}

#[contract]
pub struct StellarInvoiceContract;


impl StellarInvoiceContract {
    fn get_counter(env: &Env) -> u64 {
        env.storage().instance().get(&DataKey::Counter).unwrap_or(0_u64)
    }

    fn set_counter(env: &Env, v: u64) {
        env.storage().instance().set(&DataKey::Counter, &v);
    }

    fn store_invoice(env: &Env, invoice: &Invoice) {
        env.storage().persistent().set(&DataKey::Invoice(invoice.id), invoice);
    }

    fn load_invoice(env: &Env, id: u64) -> Option<Invoice> {
        env.storage().persistent().get(&DataKey::Invoice(id))
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
}


#[contractimpl]
impl StellarInvoiceContract {
    // Create invoice and return id
    pub fn create_invoice(env: Env, issuer: Address, payer: Address, amount: i128, currency: Symbol, due_date: u64) -> u64 {
        // increment counter
        let mut counter = Self::get_counter(&env);
        counter += 1;
        Self::set_counter(&env, counter);

        let created_at = env.ledger().timestamp();

        let invoice = Invoice {
            id: counter,
            issuer: issuer.clone(),
            payer: payer.clone(),
            amount,
            currency: currency.clone(),
            due_date,
            status: Status::Created,
            created_at,
            paid_amount: 0,
        };

        // persist
        StellarInvoiceContract::store_invoice(&env, &invoice);
        StellarInvoiceContract::push_invoice_to_address(&env, &issuer, invoice.id);
        StellarInvoiceContract::push_invoice_to_address(&env, &payer, invoice.id);

        // emit event
        env.events().publish((symbol_short!("Invoice"), symbol_short!("Created")), (invoice.id, issuer, payer, amount, currency, due_date));

        invoice.id
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

        env.events().publish((symbol_short!("Invoice"), symbol_short!("Paid")), (invoice.id, payer, amount, invoice.paid_amount, invoice.status));
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
            env.events().publish((symbol_short!("Invoice"), symbol_short!("Overdue")), (invoice.id, invoice.due_date, ts));
        }
    }
}

// Unit tests using soroban-sdk testutils
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Env as TestEnv, Env, BytesN, Address, Symbol};

    fn addr_from_byte(env: &Env, b: u8) -> Address {
        Address::from_contract_id(&env, &BytesN::from_array(&env, &[b; 32]))
    }

    #[test]
    fn test_full_payment() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 1);
        let payer = addr_from_byte(&env, 2);
        let currency = Symbol::short("XLM");

        let id = StellarInvoiceContract::create_invoice(env.clone(), issuer.clone(), payer.clone(), 1000_i128, currency.clone(), env.ledger().timestamp() + 1000);
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

        let id = StellarInvoiceContract::create_invoice(env.clone(), issuer.clone(), payer.clone(), 1000_i128, currency.clone(), env.ledger().timestamp() + 1000);
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 400_i128);
        let inv = StellarInvoiceContract::get_invoice(env.clone(), id);
        assert_eq!(inv.paid_amount, 400_i128);
        assert_eq!(inv.status, Status::PartiallyPaid);
    }

    #[test]
    fn test_overdue_transition() {
        let env = TestEnv::default();
        let issuer = addr_from_byte(&env, 5);
        let payer = addr_from_byte(&env, 6);
        let currency = Symbol::short("XLM");

        // create invoice due immediately
        let now = env.ledger().timestamp();
        let id = StellarInvoiceContract::create_invoice(env.clone(), issuer.clone(), payer.clone(), 1000_i128, currency.clone(), now);

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

        let id = StellarInvoiceContract::create_invoice(env.clone(), issuer.clone(), payer.clone(), 1000_i128, currency.clone(), env.ledger().timestamp() + 1000);
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

        let id = StellarInvoiceContract::create_invoice(env.clone(), issuer.clone(), payer.clone(), 1000_i128, currency.clone(), env.ledger().timestamp() + 1000);
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 1000_i128);
        // second payment should panic due to overpayment
        StellarInvoiceContract::pay_invoice(env.clone(), id, payer.clone(), 1_i128);
    }
}
