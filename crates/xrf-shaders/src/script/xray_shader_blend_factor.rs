/// One factor of a blend equation, as a shader script spells it: `blend.srcalpha` and its kin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XRayShaderBlendFactor {
  Zero,
  One,
  SourceColor,
  InverseSourceColor,
  SourceAlpha,
  InverseSourceAlpha,
  DestinationAlpha,
  InverseDestinationAlpha,
  DestinationColor,
  InverseDestinationColor,
  SourceAlphaSaturated,
}

impl XRayShaderBlendFactor {
  /// The prefix every factor is written behind, since the script reads them off a table called `blend`.
  const PREFIX: &'static str = "blend.";

  /// The factor a script's name states, or `None` for a name this does not model.
  pub fn of(name: &str) -> Option<Self> {
    match name.trim().trim_start_matches(Self::PREFIX) {
      "zero" => Some(Self::Zero),
      "one" => Some(Self::One),
      "srccolor" => Some(Self::SourceColor),
      "invsrccolor" => Some(Self::InverseSourceColor),
      "srcalpha" => Some(Self::SourceAlpha),
      "invsrcalpha" => Some(Self::InverseSourceAlpha),
      "destalpha" => Some(Self::DestinationAlpha),
      "invdestalpha" => Some(Self::InverseDestinationAlpha),
      "destcolor" => Some(Self::DestinationColor),
      "invdestcolor" => Some(Self::InverseDestinationColor),
      "srcalphasat" => Some(Self::SourceAlphaSaturated),
      _ => None,
    }
  }
}
