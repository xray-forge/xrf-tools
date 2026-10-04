use crate::data::xray_surface_draw::XraySurfaceDraw;

/// The blend equation `B_PARTICLE`'s `Blending` token selects, as the deferred renderer compiles it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XrayParticleBlending {
  /// `SET`: the deferred, lit `deffer_particle` pass, cut out at the shader's own constant.
  Set,
  /// `BLEND`: source alpha over inverse source alpha.
  Blend,
  /// `ADD`: added.
  Add,
  /// `MUL`: multiplied by the destination.
  Multiply,
  /// `MUL_2X`: multiplied both ways.
  MultiplyDoubled,
  /// `ALPHA-ADD`: added, weighed by its alpha.
  AlphaAdd,
}

impl XrayParticleBlending {
  /// The token the class writes the blend equation under.
  pub(crate) const PROPERTY: &'static str = "Blending";

  /// The equation an authored index selects, or `None` for an index the class does not define.
  pub(crate) const fn of(selected: u32) -> Option<Self> {
    match selected {
      0 => Some(Self::Set),
      1 => Some(Self::Blend),
      2 => Some(Self::Add),
      3 => Some(Self::Multiply),
      4 => Some(Self::MultiplyDoubled),
      5 => Some(Self::AlphaAdd),
      _ => None,
    }
  }

  /// How a particle of this equation is drawn; every forward pass tests against zero, whatever `Alpha ref` says.
  pub(crate) const fn draw(self) -> XraySurfaceDraw {
    match self {
      // Written into the g-buffer and lit there, as every cut-out surface is.
      Self::Set => XraySurfaceDraw::AlphaTested {
        reference: XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
      },
      Self::Blend => XraySurfaceDraw::Blended { reference: 0 },
      Self::Add => XraySurfaceDraw::Added {
        is_weighted: false,
        reference: 0,
      },
      Self::Multiply => XraySurfaceDraw::Multiplied { is_doubled: false },
      Self::MultiplyDoubled => XraySurfaceDraw::Multiplied { is_doubled: true },
      Self::AlphaAdd => XraySurfaceDraw::Added {
        is_weighted: true,
        reference: 0,
      },
    }
  }
}
