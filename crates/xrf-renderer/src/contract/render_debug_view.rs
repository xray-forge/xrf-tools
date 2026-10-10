use serde::{Deserialize, Serialize};

/// Which picture a viewport shows: its finished frame, or one of the targets the frame was built from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDebugView {
  /// The finished frame.
  #[default]
  Final,
  /// The G-buffer's albedo, raw.
  Albedo,
  /// The gloss the albedo target carries in its alpha.
  Gloss,
  /// The view space normal, remapped to colour.
  Normal,
  /// The baked hemisphere occlusion.
  Hemi,
  /// The baked sun occlusion.
  Sun,
  /// The lighting model slice, `(class + 0.5) / 4`.
  Material,
  /// View distance, logarithmic: near dark, far light.
  Depth,
  /// What the sun and the lights accumulated.
  Light,
  /// The screen's occlusion, white where it is off.
  AmbientOcclusion,
  /// How far each surface moved on the screen since the last frame: red across, green down, grey still.
  Motion,
  /// The indirect light as it would light a white surface, black where it is not gathered.
  IndirectLight,
  /// The screen-space reflections as traced: what a ray met in its colour, deep blue where it met nothing and the cube
  /// stands, black where nothing is traced.
  Reflections,
  /// How each reflection ray ended, one colour a reason (`common/reflection_end.wgsl`); black where nothing is traced.
  ReflectionEnds,
  /// How far parallax moved each surface's texture: across in red and down in green about a half, out to the march's
  /// reach, over full blue; a half in both where a surface takes parallax but stands past its reach; every other
  /// surface a dim grey of its own brightness.
  Parallax,
  /// Which surfaces are bumped: a bumped one's normal in its own tangent space as a normal map colours it, lavender
  /// where flat, a terrain's by its four details' bumps; a surface without a bump red by its brightness.
  Bumps,
}

impl RenderDebugView {
  /// The number the present shader tells it by.
  pub fn get_index(self) -> u32 {
    self as u32
  }
}
