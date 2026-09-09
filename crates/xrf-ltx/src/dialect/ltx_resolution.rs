use std::collections::HashMap;
use std::sync::Arc;

use fxhash::FxBuildHasher;

use crate::ltx::Ltx;

/// What resolving one root answers with.
///
/// Carries more than the resolved document because a caller may have to account for a value: in a modded install the
/// only way to explain one is to name the file that won it, and in any tree a field a section never wrote is worth
/// distinguishing from one it did.
#[derive(Debug, Default)]
pub struct LtxResolution {
  pub ltx: Ltx,
  /// Where each resolved field came from, when the resolution was asked for it.
  ///
  /// Empty unless [`crate::LtxResolveRequest::with_provenance`] was passed, and empty is not a claim that every field
  /// was declared where it sits - it is the absence of a record. See [`LtxProvenance`].
  pub provenance: LtxProvenance,
  /// What the dialect found worth saying that is not a failure.
  pub diagnostics: Vec<LtxResolutionDiagnostic>,
}

impl LtxResolution {
  /// A resolution with nothing recorded beside it, which is what an unasked resolve answers.
  pub fn new_plain(ltx: Ltx) -> Self {
    Self {
      diagnostics: Vec::new(),
      ltx,
      provenance: LtxProvenance::default(),
    }
  }

  /// Where one field came from, when this resolution recorded it.
  pub fn get_origin(&self, section: &str, key: &str) -> Option<&LtxFieldOrigin> {
    self.provenance.get(section, key)
  }
}

/// Where every resolved field came from, grouped by the section holding it.
///
/// Nested rather than keyed by a `(section, key)` pair, for two reasons. A pair key cannot be looked up from two
/// `&str` without building the owned halves first, so every read would allocate; and the question a caller actually
/// asks is "explain this section", which a flat map answers only by scanning. Both levels key on `Arc<str>`, which
/// borrows as `str`, so a lookup allocates nothing.
///
/// Storage is private: this is a record with two questions, not a map that happens to hold origins.
///
/// One known limitation, pinned by `ltx::tests::field_provenance`: the root section is the only section that merges
/// across files instead of colliding, so when two files of one tree both write root fields, every root field reports
/// the first file merged. Every named section is exact, because `Ltx::merge_sections_from` refuses a duplicate of one.
#[derive(Debug, Default)]
pub struct LtxProvenance {
  sections: HashMap<Arc<str>, HashMap<Arc<str>, LtxFieldOrigin, FxBuildHasher>, FxBuildHasher>,
}

impl LtxProvenance {
  /// Whether nothing was recorded, which is the state of any resolution that was not asked for provenance.
  pub fn is_empty(&self) -> bool {
    self.sections.is_empty()
  }

  /// How many sections carry a record.
  pub fn len(&self) -> usize {
    self.sections.len()
  }

  /// Where one field came from.
  pub fn get(&self, section: &str, key: &str) -> Option<&LtxFieldOrigin> {
    self.sections.get(section)?.get(key)
  }

  /// Every recorded field of one section, in no particular order.
  ///
  /// Answers an empty iterator both for a section nothing recorded and for a section that does not exist, because a
  /// caller holding the resolved section already knows which it is looking at.
  pub fn iter_section(&self, section: &str) -> impl Iterator<Item = (&str, &LtxFieldOrigin)> {
    self
      .sections
      .get(section)
      .into_iter()
      .flat_map(|fields| fields.iter().map(|(key, origin)| (&**key, origin)))
  }

  /// Records where one field came from, replacing any earlier answer for it.
  ///
  /// Public for the same reason [`crate::Section::set_origin`] is: a dialect lives in another crate and has to fill in
  /// what it resolved. Reading stays through [`Self::get`] and [`Self::iter_section`].
  pub fn insert(&mut self, section: Arc<str>, key: Arc<str>, origin: LtxFieldOrigin) {
    self.sections.entry(section).or_default().insert(key, origin);
  }

  /// Records a whole section's fields at once, which is how a dialect fills this in.
  pub(crate) fn insert_section(&mut self, section: Arc<str>, fields: HashMap<Arc<str>, LtxFieldOrigin, FxBuildHasher>) {
    self.sections.insert(section, fields);
  }

  /// Give back the growth room the record no longer needs.
  pub(crate) fn shrink_to_fit(&mut self) {
    self.sections.shrink_to_fit();

    for fields in self.sections.values_mut() {
      fields.shrink_to_fit();
    }
  }
}

/// How one resolved field came to hold the value it holds.
///
/// The three variants are three different fixes for a value someone disagrees with: edit this section, edit the
/// ancestor that declares it, or edit the patch file that overrode it. Collapsing them into "which file" would lose
/// the middle one, because an inherited field's file is a file whose text never mentions the section asking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LtxFieldOrigin {
  /// Written in the body of the section that holds it.
  ///
  /// `file` is the config whose header declared that section, or nothing where no dialect stamped one.
  Declared { file: Option<Arc<str>> },
  /// Copied in by inheritance from the section that writes it.
  ///
  /// `section` is where the value is actually written, not the parent named in the header: with `[c]:b` and `[b]:a`,
  /// a field written in `a` reports `a`, because `b` is a step on the path rather than the answer.
  Inherited { section: Arc<str>, file: Option<Arc<str>> },
  /// Won a load-order contest, under a dialect that ranks statements rather than reading them in order.
  ///
  /// Not only a patch: an unpatched base field under such a dialect reports its own file, at its own depth, with an
  /// empty operation. Naming it for the contest rather than for patching is what keeps that case honest.
  Loaded {
    /// Logical path of the winning file.
    file: Arc<str>,
    /// Load rank of the winning statement. Negative means a patch file outranking the base tree.
    depth: i32,
    /// How the value was written, spelled as its prefix: empty for a plain assignment, `>` for a list append.
    operation: Box<str>,
  },
}

impl LtxFieldOrigin {
  /// The config the winning value is written in, where one is known.
  pub fn get_file(&self) -> Option<&str> {
    match self {
      Self::Declared { file } | Self::Inherited { file, .. } => file.as_deref(),
      Self::Loaded { file, .. } => Some(file),
    }
  }

  /// The section the value is written in, for a field this one inherited.
  pub fn get_inherited_from(&self) -> Option<&str> {
    match self {
      Self::Inherited { section, .. } => Some(section),
      _ => None,
    }
  }

  /// Whether the field arrived by inheritance rather than being written where it sits.
  pub fn is_inherited(&self) -> bool {
    matches!(self, Self::Inherited { .. })
  }

  /// How the winning value was written, for a dialect that ranks statements.
  pub fn get_operation(&self) -> Option<&str> {
    match self {
      Self::Loaded { operation, .. } => Some(operation),
      _ => None,
    }
  }

  /// The load rank of the winning statement, for a dialect that ranks them.
  pub fn get_depth(&self) -> Option<i32> {
    match self {
      Self::Loaded { depth, .. } => Some(*depth),
      _ => None,
    }
  }
}

/// Something a dialect wants said about a config tree, short of refusing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LtxResolutionDiagnostic {
  pub section: String,
  pub file: Option<String>,
  pub message: String,
  /// What the engine does with the same input, where that differs from reporting it.
  ///
  /// The reason these are worth emitting at all: a modder cannot see what the game silently drops, so a warning that
  /// says "the game loads this and says nothing" is more use than the same warning without it.
  pub engine_behaviour: Option<String>,
}
