use xrf_material::XraySurfaceDraw;

/// The blend equation a particle effect composites its sprite with, one pipeline each (`CBlender_Particle`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParticleBlend {
  /// Written over what is behind it.
  Opaque,
  /// `SET`: cut out at its reference and written over.
  CutOut,
  /// `BLEND`: source alpha over its inverse.
  Blend,
  /// `ADD`: added.
  Add,
  /// `ALPHA-ADD`: added, weighed by its alpha.
  AlphaAdd,
  /// `MUL`: multiplied by what is behind it.
  Multiply,
  /// `MUL_2X`: multiplied both ways.
  MultiplyDoubled,
}

impl ParticleBlend {
  /// Every equation, in the order the pass keeps their pipelines.
  pub const ALL: [ParticleBlend; 7] = [
    ParticleBlend::Opaque,
    ParticleBlend::CutOut,
    ParticleBlend::Blend,
    ParticleBlend::Add,
    ParticleBlend::AlphaAdd,
    ParticleBlend::Multiply,
    ParticleBlend::MultiplyDoubled,
  ];

  /// The equation a surface draws its colour with, none for one drawing no colour of its own: an `l_special` alone, or
  /// water no sprite is.
  pub fn of(draw: XraySurfaceDraw) -> Option<Self> {
    match draw {
      XraySurfaceDraw::Opaque => Some(Self::Opaque),
      XraySurfaceDraw::AlphaTested { .. } => Some(Self::CutOut),
      XraySurfaceDraw::Blended { .. } => Some(Self::Blend),
      XraySurfaceDraw::Added { is_weighted, .. } => Some(if is_weighted { Self::AlphaAdd } else { Self::Add }),
      XraySurfaceDraw::Multiplied { is_doubled } => Some(if is_doubled {
        Self::MultiplyDoubled
      } else {
        Self::Multiply
      }),
      XraySurfaceDraw::Invisible | XraySurfaceDraw::Water { .. } => None,
    }
  }

  /// Its pipeline's index among [`Self::ALL`].
  pub fn get_index(self) -> usize {
    self as usize
  }

  /// The colour equation over the scene; the scene's alpha is left as it is.
  pub fn get_blend_state(self) -> wgpu::BlendState {
    let color = |src_factor: wgpu::BlendFactor, dst_factor: wgpu::BlendFactor| wgpu::BlendComponent {
      src_factor,
      dst_factor,
      operation: wgpu::BlendOperation::Add,
    };
    let kept: wgpu::BlendComponent = color(wgpu::BlendFactor::Zero, wgpu::BlendFactor::One);

    wgpu::BlendState {
      color: match self {
        Self::Opaque | Self::CutOut => color(wgpu::BlendFactor::One, wgpu::BlendFactor::Zero),
        Self::Blend => color(wgpu::BlendFactor::SrcAlpha, wgpu::BlendFactor::OneMinusSrcAlpha),
        Self::Add => color(wgpu::BlendFactor::One, wgpu::BlendFactor::One),
        Self::AlphaAdd => color(wgpu::BlendFactor::SrcAlpha, wgpu::BlendFactor::One),
        Self::Multiply => color(wgpu::BlendFactor::Dst, wgpu::BlendFactor::Zero),
        Self::MultiplyDoubled => color(wgpu::BlendFactor::Dst, wgpu::BlendFactor::Src),
      },
      alpha: kept,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_material::XraySurfaceDraw;

  use super::ParticleBlend;

  #[test]
  fn takes_each_draw_to_its_equation_and_none_for_no_colour() {
    assert_eq!(
      ParticleBlend::of(XraySurfaceDraw::Added {
        is_weighted: true,
        reference: 0
      }),
      Some(ParticleBlend::AlphaAdd)
    );
    assert_eq!(
      ParticleBlend::of(XraySurfaceDraw::Blended { reference: 0 }),
      Some(ParticleBlend::Blend)
    );
    assert_eq!(ParticleBlend::of(XraySurfaceDraw::Invisible), None);

    for (index, blend) in ParticleBlend::ALL.iter().enumerate() {
      assert_eq!(blend.get_index(), index);
    }
  }
}
