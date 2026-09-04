use std::sync::Arc;

use fxhash::FxBuildHasher;
use indexmap::IndexMap;

use crate::ltx::Section;
use crate::scheme::{LtxFieldScheme, LtxSectionScheme};

/// A resolved section's fields, in the order they were written.
///
/// `Arc<str>` rather than `String` or `Box<str>`, for two reasons that compound. A resolved field is never appended to,
/// so the growth capacity a `String` carries is dead weight; and inheritance copies a parent's fields into every child
/// that inherits it, so the same text is stored over and over - 293,441 resolved fields on a vanilla tree carry 10,583
/// distinct key names, a 28x repetition. Sharing turns each of those copies into a refcount bump.
pub type SectionData = IndexMap<Arc<str>, Arc<str>, FxBuildHasher>;

pub type LtxIncluded = Vec<String>;

/// A resolved config's sections, in the order they were declared.
pub type LtxSections = IndexMap<String, Section, FxBuildHasher>;

pub type LtxSectionSchemes = IndexMap<String, LtxSectionScheme, FxBuildHasher>;

pub type LtxSectionFieldSchemes = IndexMap<String, LtxFieldScheme, FxBuildHasher>;
