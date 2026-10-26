//! What a build read: the inputs a recording search resolved, the revision each had, and the revisions a VFS snapshot
//! keeps.

mod xray_input;
mod xray_input_answer;
mod xray_input_kind;
mod xray_input_record;
mod xray_revision;
mod xray_revisions;
mod xray_vfs_input_answer;

pub use xray_input::XrayInput;
pub use xray_input_answer::XrayInputAnswer;
pub use xray_input_kind::XrayInputKind;
pub use xray_input_record::XrayInputRecord;
pub use xray_revision::XrayRevision;
pub(crate) use xray_revisions::XrayRevisions;
