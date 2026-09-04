use fxhash::FxBuildHasher;
use indexmap::IndexMap;

use crate::ltx::Section;
use crate::scheme::{LtxFieldScheme, LtxSectionScheme};

pub type SectionData = IndexMap<String, String, FxBuildHasher>;

pub type LtxIncluded = Vec<String>;

pub type LtxSections = IndexMap<String, Section, FxBuildHasher>;

pub type LtxSectionSchemes = IndexMap<String, LtxSectionScheme, FxBuildHasher>;

pub type LtxSectionFieldSchemes = IndexMap<String, LtxFieldScheme, FxBuildHasher>;
