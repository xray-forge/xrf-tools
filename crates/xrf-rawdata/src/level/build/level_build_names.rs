use std::borrow::Cow;
use std::collections::HashMap;

use xrf_error::{XrfError, XrfResult};

/// One name table of a build: each name kept once, in the order first named, its index found by a hash of its key.
///
/// Shaders are told apart exactly and textures without ASCII case, as the editor compares them; the table keeps the
/// spelling a name was first given.
pub(crate) struct LevelBuildNames {
  names: Vec<String>,
  indexes: HashMap<String, u16>,
  is_case_blind: bool,
}

impl LevelBuildNames {
  /// The index past the last a table may hand out, `u16(-1)`, which a material's compiler shader takes for none.
  pub(crate) const NO_INDEX: u16 = u16::MAX;

  /// A table telling names apart exactly.
  pub(crate) fn exact() -> Self {
    Self::new(false)
  }

  /// A table telling names apart without ASCII case.
  pub(crate) fn case_blind() -> Self {
    Self::new(true)
  }

  fn new(is_case_blind: bool) -> Self {
    Self {
      names: Vec::new(),
      indexes: HashMap::new(),
      is_case_blind,
    }
  }

  /// The index of `name`, appended where the table holds none like it.
  ///
  /// # Errors
  ///
  /// Returns an error when the table would outgrow its `u16` indices.
  pub(crate) fn intern(&mut self, name: &str) -> XrfResult<u16> {
    // Most names repeat, so a name with nothing to fold is looked up as given, with no copy.
    let key: Cow<'_, str> = if self.is_case_blind && name.bytes().any(|byte| byte.is_ascii_uppercase()) {
      Cow::Owned(name.to_ascii_lowercase())
    } else {
      Cow::Borrowed(name)
    };

    if let Some(index) = self.indexes.get(key.as_ref()) {
      return Ok(*index);
    }

    let index: u16 = Self::get_index(self.names.len())?;

    self.names.push(name.to_owned());
    self.indexes.insert(key.into_owned(), index);

    Ok(index)
  }

  /// The names in the order first named.
  pub(crate) fn get_names(&self) -> &[String] {
    &self.names
  }

  /// `index` as a table index, refused where it reaches [`Self::NO_INDEX`].
  ///
  /// # Errors
  ///
  /// Returns an error when `index` is past the last a table may hand out.
  pub(crate) fn get_index(index: usize) -> XrfResult<u16> {
    u16::try_from(index)
      .ok()
      .filter(|index| *index != Self::NO_INDEX)
      .ok_or_else(|| XrfError::new_invalid_error("A build's table holds more entries than its u16 indices number"))
  }
}
