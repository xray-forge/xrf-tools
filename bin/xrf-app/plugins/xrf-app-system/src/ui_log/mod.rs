//! The webview's log, written to this run's UI log file: the level it is filtered at, so a filtered record is never
//! sent, and the batches the webview sends.

mod batch;
pub(crate) mod commands;
mod request;
#[cfg(test)]
mod tests;

pub(crate) use batch::UiLogBatch;
pub(crate) use request::{UiLogLevel, UiLogRecord};
