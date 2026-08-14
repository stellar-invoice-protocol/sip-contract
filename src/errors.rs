use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum InvoiceError {
    /// No invoice exists for the given id.
    InvoiceNotFound = 1,
    /// The caller is not the designated payer on this invoice.
    UnauthorizedPayer = 2,
    /// The invoice is in a terminal state (Paid or Cancelled) and cannot be paid.
    InvoiceNotPayable = 3,
    /// The payment would push paid_amount above the invoice amount.
    OverpaymentNotAllowed = 4,
    /// paid_amount + payment amount overflowed i128.
    PaidAmountOverflow = 5,
    /// Caller is not the issuer; only the issuer may cancel.
    OnlyIssuerCanCancel = 6,
    /// Invoice has been fully paid and cannot be cancelled.
    CannotCancelPaidInvoice = 7,
    /// amount argument is zero or negative.
    InvalidAmount = 8,
    /// due_date is not strictly in the future relative to the current ledger timestamp.
    InvalidDueDate = 9,
}
