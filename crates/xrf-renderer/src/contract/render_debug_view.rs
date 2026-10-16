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
  /// The lighting model slice the G-buffer keeps (`material_slice`).
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
  /// The heights parallax reads: a terrain's details' weighed by its mask, an installation's own or derived from their
  /// bumps, and a steep surface's, grey from black low to white high; every other surface a dim blue.
  Heights,
  /// The scene's luminance in stops about middle grey, one colour a stop: blues below, grey at middle grey, green to
  /// red above, white six stops over and black six under.
  Luminance,
  /// The albedo's luminance read as linear light: grey within the plausible 0.03 to 0.9, blue below, red above.
  AlbedoRange,
  /// How many of the static scene's surfaces were drawn over each pixel, hidden or not: black none, then blue through
  /// green and yellow to red at eight.
  Overdraw,
  /// Which of the sun's cascades shades each pixel, a tint a cascade from the nearest, darker where it shadows, grey
  /// past them all; striped within the reach the grass's cards cast their shadows to.
  Cascades,
  /// How many lamps the cluster each pixel falls in lists: black none, then blue through green and yellow to red at
  /// sixteen.
  LampsPerCluster,
  /// The mip a surface's base texture is read at: green its own size, blue where it is magnified, yellow to red as
  /// smaller mips are read; grey where a surface has no base texture.
  SampledMip,
  /// FSR's masks at each pixel's drawn texel: how reactive it is (moving apart from what it lands on, or a plant the
  /// wind moved) in red, how disoccluded in green, how far its shading changed in blue.
  FsrMasks,
  /// How far FSR trusts each pixel's history, from black for none to white; red where it starts afresh.
  FsrHistoryTrust,
  /// FSR's thin-feature locks: each pixel's lock in green, full where it holds the history against clamping; red
  /// where a new lock was found this frame.
  FsrLocks,
  /// How strongly FSR finds each pixel's luma flickering back to an earlier frame's, which lets its history hold.
  FsrLumaInstability,
  /// How much of each pixel FSR takes from this frame rather than its history, from black for none to white for all,
  /// as its square root so a frame's usual small share reads.
  FsrFrameShare,
}

impl RenderDebugView {
  /// The number the present shader tells it by.
  pub fn get_index(self) -> u32 {
    self as u32
  }

  /// Whether it shows FSR's own targets, which FSR resolves for whatever antialiasing the view asks for.
  pub fn is_fsr(self) -> bool {
    matches!(
      self,
      Self::FsrMasks | Self::FsrHistoryTrust | Self::FsrLocks | Self::FsrLumaInstability | Self::FsrFrameShare
    )
  }
}
