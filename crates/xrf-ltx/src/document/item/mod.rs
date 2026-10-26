//! One statement of a document: what it is, where it was written, and the DLTX operation it carries.

pub(crate) mod ltx_item;
pub(crate) mod ltx_item_kind;
pub(crate) mod ltx_key_operation;
pub(crate) mod ltx_section_operation;
pub(crate) mod ltx_span;

pub use crate::document::item::ltx_item::LtxItem;
pub use crate::document::item::ltx_item_kind::LtxItemKind;
pub use crate::document::item::ltx_key_operation::LtxKeyOperation;
pub use crate::document::item::ltx_section_operation::LtxSectionOperation;
pub use crate::document::item::ltx_span::LtxSpan;
