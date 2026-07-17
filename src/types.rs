use soroban_sdk::{contracttype, Address, Symbol};

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