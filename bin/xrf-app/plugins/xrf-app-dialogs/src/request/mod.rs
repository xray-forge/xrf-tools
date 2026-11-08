//! What the dialogs commands taking more than a few arguments were asked to do.

mod dialogs_open_request;
mod dialogs_read_request;

pub(crate) use dialogs_open_request::DialogsOpenRequest;
pub(crate) use dialogs_read_request::DialogsReadRequest;
