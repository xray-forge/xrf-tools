use crate::data::xray_surface_draw::XraySurfaceDraw;

/// The blend equation `B_SCREEN_SET`'s `Blending` token selects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XrayScreenSetBlending {
  /// `SET`: written straight over what is behind it, reading no alpha at all.
  Set,
  /// `BLEND`, `BLEND (2r)`, `BLEND (4r)`: source alpha over inverse source alpha, tested.
  Blend,
  /// `ADD`: added, untested.
  Add,
  /// `MUL`: multiplied by the destination, untested.
  Multiply,
  /// `MUL_2X`, `MUL_2X (B^D)`: multiplied both ways, untested.
  MultiplyDoubled,
  /// `ALPHA-ADD`: added, tested.
  AlphaAdd,
  /// `SET (2r)`: written over, tested against nothing, which is what a reference of zero comes to.
  SetTested,
}

impl XrayScreenSetBlending {
  /// The token the class writes the blend equation under.
  pub(crate) const PROPERTY: &'static str = "Blending";

  /// The equation an authored index selects, or `None` for an index no build of the class defines.
  pub(crate) const fn of(selected: u32) -> Option<Self> {
    match selected {
      0 => Some(Self::Set),
      1 | 8 | 9 => Some(Self::Blend),
      2 => Some(Self::Add),
      3 => Some(Self::Multiply),
      4 | 6 => Some(Self::MultiplyDoubled),
      5 => Some(Self::AlphaAdd),
      7 => Some(Self::SetTested),
      _ => None,
    }
  }

  /// How a surface of this equation is drawn, given the `Alpha ref` its blender authored.
  pub(crate) const fn draw(self, reference: u8) -> XraySurfaceDraw {
    match self {
      // Blending off entirely, and the tested variant tests against zero, which discards nothing.
      Self::Set | Self::SetTested => XraySurfaceDraw::Opaque,
      Self::Blend => XraySurfaceDraw::Blended { reference },
      Self::Add => XraySurfaceDraw::Added {
        is_weighted: false,
        reference: 0,
      },
      Self::AlphaAdd => XraySurfaceDraw::Added {
        is_weighted: true,
        reference,
      },
      Self::Multiply => XraySurfaceDraw::Multiplied { is_doubled: false },
      Self::MultiplyDoubled => XraySurfaceDraw::Multiplied { is_doubled: true },
    }
  }
}
