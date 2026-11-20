//! The window as its page drives it: shown once the page has drawn, closed once the page agrees, and the page's
//! decision about leaving, which only the page knows.

pub(crate) mod commands;
mod request;

pub(crate) use request::LeaveDecision;
