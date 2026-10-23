//! What a build read: the inputs a recording probe resolved, the revision each had, and the revisions a world keeps.

mod xray_input;
mod xray_input_answer;
mod xray_input_record;
mod xray_revision;

pub use xray_input::{XrayInput, XrayInputKind, XraySearch};
pub use xray_input_answer::XrayInputAnswer;
pub use xray_input_record::XrayInputRecord;
pub use xray_revision::XrayRevision;
pub(crate) use xray_revision::XrayRevisions;
