#![no_std]

pub mod contract;
pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

pub use contract::StellarInvoiceContract;
pub use errors::InvoiceError;
pub use types::{Invoice, Status};

#[cfg(test)]
mod test;
