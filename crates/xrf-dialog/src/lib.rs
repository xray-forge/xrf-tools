//! Reading S.T.A.L.K.E.R. dialog XML.
//!
//! The tree follows the data: an `element` is one written child, a `phrase` is one line, a `dialog`
//! is one conversation, and a `file` is one document together with the text its ranges address.
//! Nothing here validates a dialog or resolves a translation key — those need a project around the
//! file, and belong above this crate.

pub(crate) mod constants;
pub(crate) mod dialog;
pub(crate) mod element;
pub(crate) mod encoding;
pub(crate) mod file;
pub(crate) mod issue;
pub(crate) mod phrase;

pub use crate::dialog::Dialog;
pub use crate::element::{DialogElement, DialogElementKind};
pub use crate::file::DialogFile;
pub use crate::issue::{DialogParseIssue, DialogParseIssueKind};
pub use crate::phrase::DialogPhrase;

#[cfg(test)]
mod tests;
