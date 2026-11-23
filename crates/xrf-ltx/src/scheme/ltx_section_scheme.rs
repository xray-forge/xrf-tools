use xrf_collections::OrderedMap;

use crate::scheme::LtxFieldScheme;

/// The field schemes one section scheme declares, keyed by field name in declared order.
pub type LtxSectionFieldSchemes = OrderedMap<String, LtxFieldScheme>;

/// Every section scheme a project declares, keyed by scheme name in declared order.
pub type LtxSectionSchemes = OrderedMap<String, LtxSectionScheme>;

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
