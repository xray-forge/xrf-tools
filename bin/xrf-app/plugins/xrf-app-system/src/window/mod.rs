//! The window as its page sees it: the moment the page has drawn its first screen, a close the page asked about
//! first and agreed to, and what the page decided about leaving, which only the page knows.

pub(crate) mod commands;
mod request;

pub(crate) use request::LeaveDecision;
