use fxhash::FxBuildHasher;
use indexmap::IndexMap;

use crate::scheme::LtxFieldScheme;

/// The field schemes one section scheme declares, keyed by field name in declared order.
pub type LtxSectionFieldSchemes = IndexMap<String, LtxFieldScheme, FxBuildHasher>;

/// Every section scheme a project declares, keyed by scheme name in declared order.
pub type LtxSectionSchemes = IndexMap<String, LtxSectionScheme, FxBuildHasher>;

/// Scheme definition for section in LTX file.
#[derive(Clone, Debug, Default)]
pub struct LtxSectionScheme {
  pub name: String,
  pub is_strict: bool,
  pub fields: LtxSectionFieldSchemes,
}

impl LtxSectionScheme {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.into(),
      is_strict: false,
      fields: Default::default(),
    }
  }
}
