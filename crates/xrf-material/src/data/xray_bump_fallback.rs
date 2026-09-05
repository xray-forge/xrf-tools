/// What `CRender::texture_load` binds in place of a bump input it cannot find (`Texture.cpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrayBumpFallback {
  /// `ed\ed_dummy_bump`, a flat `normal.gloss` for an absent bump.
  DummyBump,
  /// `ed\ed_dummy_bump#`, a flat `normal_error.height` for an absent companion.
  DummyCompanion,
  /// `ed\ed_not_existing_texture`, for an absent name the renderer does not recognise as a bump.
  NotExisting,
}

impl XrayBumpFallback {
  /// The substring the renderer tests before choosing a dummy.
  const BUMP_MARKER: &'static str = "_bump";

  /// The fallback the renderer picks for one input, by the name it was asked for.
  ///
  /// The companion's name is the bump's with `#` appended, so it carries the marker whenever the bump does; the flag
  /// only selects which dummy, since `texture_load` tests `_bump#` before `_bump`.
  pub fn for_input(reference: &str, is_companion: bool) -> Self {
    if !reference.contains(Self::BUMP_MARKER) {
      Self::NotExisting
    } else if is_companion {
      Self::DummyCompanion
    } else {
      Self::DummyBump
    }
  }

  /// The fallback a substituted resolution names, or `None` for a reference that is not one of the three.
  pub fn of_reference(reference: &str) -> Option<Self> {
    [Self::DummyBump, Self::DummyCompanion, Self::NotExisting]
      .into_iter()
      .find(|fallback| fallback.reference() == reference)
  }

  /// The engine path, without extension, the way the renderer spells it.
  pub const fn reference(self) -> &'static str {
    match self {
      Self::DummyBump => "ed\\ed_dummy_bump",
      Self::DummyCompanion => "ed\\ed_dummy_bump#",
      Self::NotExisting => "ed\\ed_not_existing_texture",
    }
  }

  /// Whether the bump shader still runs over this substitute, which is what makes a dummy a silent cost.
  pub const fn is_dummy(self) -> bool {
    matches!(self, Self::DummyBump | Self::DummyCompanion)
  }
}
