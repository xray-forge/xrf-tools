#![doc = include_str!("../README.md")]

pub(crate) mod conversation;
pub(crate) mod dialog_file;
pub(crate) mod dialog_source;
pub(crate) mod partner;
pub(crate) mod project;

pub use crate::conversation::dialog::Dialog;
pub use crate::conversation::dialog_element::DialogElement;
pub use crate::conversation::dialog_element_kind::DialogElementKind;
pub use crate::conversation::dialog_parse_issue::DialogParseIssue;
pub use crate::conversation::dialog_parse_issue_kind::DialogParseIssueKind;
pub use crate::conversation::dialog_phrase::DialogPhrase;
pub use crate::dialog_file::DialogFile;
pub use crate::partner::dialog_character::DialogCharacter;
pub use crate::partner::dialog_offer::DialogOffer;
pub use crate::partner::dialog_partners::DialogPartners;
pub use crate::partner::dialog_profile::DialogProfile;
pub use crate::project::descriptor::dialog_descriptor::DialogDescriptor;
pub use crate::project::descriptor::dialog_element_descriptor::DialogElementDescriptor;
pub use crate::project::descriptor::dialog_file_descriptor::DialogFileDescriptor;
pub use crate::project::descriptor::dialog_phrase_descriptor::DialogPhraseDescriptor;
pub use crate::project::descriptor::dialog_project_descriptor::DialogProjectDescriptor;
pub use crate::project::descriptor::dialog_reference_descriptor::DialogReferenceDescriptor;
pub use crate::project::descriptor::dialog_summary_descriptor::DialogSummaryDescriptor;
pub use crate::project::dialog_finding::DialogFinding;
pub use crate::project::dialog_project::DialogProject;
pub use crate::project::dialog_project_file::DialogProjectFile;
pub use crate::project::dialog_project_layout::DialogProjectLayout;
pub use crate::project::dialog_project_mode::DialogProjectMode;
pub use crate::project::text::dialog_text_index::DialogTextIndex;
pub use crate::project::text::dialog_text_language::DialogTextLanguage;

#[cfg(test)]
mod tests;
