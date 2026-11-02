//! One gesture's writes and deletes, addressed through the search they were made from.

mod xray_transaction;
mod xray_transaction_target;
mod xray_transaction_write;

pub use xray_transaction::XrayTransaction;
pub(crate) use xray_transaction_target::XrayTransactionTarget;
pub(crate) use xray_transaction_write::XrayTransactionWrite;
