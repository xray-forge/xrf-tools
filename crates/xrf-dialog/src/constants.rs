//! The element and attribute names X-Ray dialog XML is written with.

pub(crate) const ROOT_ELEMENT: &str = "game_dialogs";
pub(crate) const DIALOG_ELEMENT: &str = "dialog";
pub(crate) const PHRASE_ELEMENT: &str = "phrase";
pub(crate) const PHRASE_LIST_ELEMENT: &str = "phrase_list";

pub(crate) const ID_ATTRIBUTE: &str = "id";
pub(crate) const PRIORITY_ATTRIBUTE: &str = "priority";

/// Attributes a `dialog` may carry.
pub(crate) const DIALOG_ATTRIBUTES: &[&str] = &[ID_ATTRIBUTE, PRIORITY_ATTRIBUTE];

/// Attributes a `phrase` may carry.
pub(crate) const PHRASE_ATTRIBUTES: &[&str] = &[ID_ATTRIBUTE];
