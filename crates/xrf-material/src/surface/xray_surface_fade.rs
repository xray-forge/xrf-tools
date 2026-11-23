use serde::Serialize;

/// How a script's base pass draws a model unlit and faded by how it faces the eye: `model_distort` or
/// `model_distort_inv` colouring each vertex by its fade, into a particle program that draws by it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceFade {
  /// Whether it fades in where it faces the eye, `|dot(view, normal)|` (`model_distort_inv`), rather than at its rim,
  /// `1 - |dot(view, normal)|` (`model_distort`).
  pub is_facing: bool,
  /// Whether it draws its faded alpha alone, a grey in every channel (`particle_alphaonly`), rather than its texel times
  /// the fade (`particle`, `particle_hard`).
  pub is_alpha_only: bool,
}

impl XraySurfaceFade {
  /// The vertex programs that colour a model by its fade from the eye: the rim's, and the inverse facing it.
  const RIM_PROGRAM: &'static str = "model_distort";
  const FACING_PROGRAM: &'static str = "model_distort_inv";

  /// The pixel programs that draw a texel by its vertex's colour alone, unlit.
  const TEXEL_PROGRAMS: [&'static str; 2] = ["particle", "particle_hard"];

  /// The pixel program that draws the vertex's alpha times the texel's alone, in every channel.
  const ALPHA_ONLY_PROGRAM: &'static str = "particle_alphaonly";

  /// The fade a pass beginning these programs draws, named without regard to case as the engine finds its programs;
  /// `None` for a pass that does not fade, or fades into a program that lights or distorts.
  pub(crate) fn of_programs(vertex_program: &str, pixel_program: &str) -> Option<Self> {
    let is_facing: bool = if vertex_program.eq_ignore_ascii_case(Self::FACING_PROGRAM) {
      true
    } else if vertex_program.eq_ignore_ascii_case(Self::RIM_PROGRAM) {
      false
    } else {
      return None;
    };
    let is_alpha_only: bool = if pixel_program.eq_ignore_ascii_case(Self::ALPHA_ONLY_PROGRAM) {
      true
    } else if Self::TEXEL_PROGRAMS
      .iter()
      .any(|program| pixel_program.eq_ignore_ascii_case(program))
    {
      false
    } else {
      return None;
    };

    Some(Self {
      is_facing,
      is_alpha_only,
    })
  }
}
