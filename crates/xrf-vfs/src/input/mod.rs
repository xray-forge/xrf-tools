//! What a build read: the inputs a recording search resolved, the revision each had, and the revisions a VFS snapshot
//! keeps.

mod xray_input;
mod xray_input_answer;
mod xray_input_record;
mod xray_revision;

pub use xray_input::{XrayInput, XrayInputKind, XraySearchKey};
pub use xray_input_answer::XrayInputAnswer;
pub use xray_input_record::XrayInputRecord;
pub use xray_revision::XrayRevision;
pub(crate) use xray_revision::XrayRevisions;
