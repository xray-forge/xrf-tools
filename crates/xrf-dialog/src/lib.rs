#![doc = include_str!("../README.md")]

pub(crate) mod constants;
pub(crate) mod dialog;
pub(crate) mod dialog_element;
pub(crate) mod dialog_element_kind;
pub(crate) mod dialog_parse_issue;
pub(crate) mod dialog_parse_issue_kind;
pub(crate) mod encoding;
pub(crate) mod file;
pub(crate) mod partner;
pub(crate) mod phrase;
pub(crate) mod project;

pub use crate::dialog::Dialog;
pub use crate::dialog_element::DialogElement;
pub use crate::dialog_element_kind::DialogElementKind;
pub use crate::dialog_parse_issue::DialogParseIssue;
pub use crate::dialog_parse_issue_kind::DialogParseIssueKind;
pub use crate::file::DialogFile;
pub use crate::partner::dialog_character::DialogCharacter;
pub use crate::partner::dialog_offer::DialogOffer;
pub use crate::partner::dialog_partners::DialogPartners;
pub use crate::partner::dialog_profile::DialogProfile;
pub use crate::phrase::DialogPhrase;
pub use crate::project::dialog_descriptor::DialogDescriptor;
pub use crate::project::dialog_element_descriptor::DialogElementDescriptor;
pub use crate::project::dialog_file_descriptor::DialogFileDescriptor;
pub use crate::project::dialog_finding::DialogFinding;
pub use crate::project::dialog_phrase_descriptor::DialogPhraseDescriptor;
pub use crate::project::dialog_project::DialogProject;
pub use crate::project::dialog_project_descriptor::DialogProjectDescriptor;
pub use crate::project::dialog_project_file::DialogProjectFile;
pub use crate::project::dialog_summary_descriptor::DialogSummaryDescriptor;
pub use crate::project::dialog_text_index::DialogTextIndex;
pub use crate::project::dialog_text_language::DialogTextLanguage;
pub use crate::project::layout::{DialogProjectLayout, detect_mode, detect_mode_in};
pub use crate::project::mode::DialogProjectMode;
pub use crate::project::reference_descriptor::DialogReferenceDescriptor;

#[cfg(test)]
mod tests;
