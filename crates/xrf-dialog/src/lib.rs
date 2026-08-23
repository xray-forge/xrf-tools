//! Reading S.T.A.L.K.E.R. dialog XML.
//!
//! The tree follows the data: an `element` is one written child, a `phrase` is one line, a `dialog`
//! is one conversation, a `file` is one document together with the text its ranges address, and
//! `project` is a whole tree of them.

pub(crate) mod constants;
pub(crate) mod dialog;
pub(crate) mod element;
pub(crate) mod encoding;
pub(crate) mod file;
pub(crate) mod issue;
pub(crate) mod phrase;
pub(crate) mod project;

pub use crate::dialog::Dialog;
pub use crate::element::{DialogElement, DialogElementKind};
pub use crate::file::DialogFile;
pub use crate::issue::{DialogParseIssue, DialogParseIssueKind};
pub use crate::phrase::DialogPhrase;
pub use crate::project::descriptor::{DialogDescriptor, DialogFileDescriptor, DialogFinding, DialogProjectDescriptor};
pub use crate::project::dialog_project::{DialogProject, DialogProjectFile};
pub use crate::project::layout::{DialogProjectLayout, detect_mode, detect_mode_in};
pub use crate::project::mode::DialogProjectMode;

#[cfg(test)]
mod tests;
