/// The naming the SDK and the renderer agree on for a bump pair.
pub struct XrayBumpNaming;

impl XrayBumpNaming {
  /// Appended to the bump name to address its companion (`uber_deffer.cpp:82`).
  pub const COMPANION_SUFFIX: &'static str = "#";

  /// The substring `texture_load` tests before choosing a dummy (`Texture.cpp`), and the suffix the SDK gives a generated
  /// bump (`NormalMapGen.cpp:831`).
  pub const BUMP_MARKER: &'static str = "_bump";

  /// The companion the engine binds beside `bump`.
  pub fn companion_of(bump: &str) -> String {
    format!("{bump}{}", Self::COMPANION_SUFFIX)
  }

  /// Whether a name is spelled as a companion, which only a companion is.
  pub fn is_companion(reference: &str) -> bool {
    reference.ends_with(Self::COMPANION_SUFFIX)
  }

  /// Whether a name follows the SDK's convention for a generated bump.
  pub fn is_conventional_bump(reference: &str) -> bool {
    reference.ends_with(Self::BUMP_MARKER)
  }

  /// Whether the renderer would recognise a missing name as a bump and substitute a dummy for it.
  pub fn carries_marker(reference: &str) -> bool {
    reference.contains(Self::BUMP_MARKER)
  }
}
