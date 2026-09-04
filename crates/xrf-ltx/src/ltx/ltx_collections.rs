use fxhash::FxBuildHasher;
use indexmap::IndexMap;

use crate::ltx::Section;
use crate::scheme::{LtxFieldScheme, LtxSectionScheme};

/// A resolved section's fields, in the order they were written.
///
/// `Box<str>` rather than `String`: a resolved field is never appended to, so the growth capacity a `String` carries
/// is eight bytes per key and per value that nothing can use. A full config sweep holds hundreds of thousands of them.
pub type SectionData = IndexMap<Box<str>, Box<str>, FxBuildHasher>;

pub type LtxIncluded = Vec<String>;

pub type LtxSections = IndexMap<String, Section, FxBuildHasher>;

pub type LtxSectionSchemes = IndexMap<String, LtxSectionScheme, FxBuildHasher>;

pub type LtxSectionFieldSchemes = IndexMap<String, LtxFieldScheme, FxBuildHasher>;
