use std::sync::Arc;

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
