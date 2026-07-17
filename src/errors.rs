use soroban_sdk::Symbol;

pub const ERR_INVOICE_NOT_FOUND: Symbol = symbol_short!("INV_NOT");
pub const ERR_UNAUTHORIZED_PAYER: Symbol = symbol_short!("UNAUTH_PAY");
pub const ERR_INVOICE_NOT_PAYABLE: Symbol = symbol_short!("NOT_PAYABLE");
pub const ERR_OVERPAYMENT: Symbol = symbol_short!("OVERPAY");
pub const ERR_ONLY_ISSUER: Symbol = symbol_short!("ONLY_ISSUER");
pub const ERR_CANNOT_CANCEL_PAID: Symbol = symbol_short!("CANT_CANCEL");