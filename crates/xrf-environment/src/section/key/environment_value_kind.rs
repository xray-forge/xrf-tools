use serde::Serialize;

/// How the engine reads a key, which decides both how it is parsed here and what a malformed value turns into.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EnvironmentValueKind {
  /// `r_float`.
  Number,
  /// `r_s32`.
  Integer,
  /// `r_fvector2`, `r_fvector3`, `r_fvector4` or a `sscanf` of its own: `most` comma-separated floats are read, and
  /// fewer than `least` is a misread.
  Vector { least: u8, most: u8 },
  /// `r_bool`: true for `on`, `yes`, `true` or `1`, false for anything else.
  Flag,
  /// `r_string`, kept as written.
  Text,
  /// A comma-separated list the engine walks with `_GetItem`.
  List,
}
