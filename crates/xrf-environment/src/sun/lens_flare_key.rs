use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::finding::EnvironmentRule;
use crate::key::{EnvironmentKey, declare_environment_keys};
use crate::section::{EnvironmentSection, EnvironmentSectionReader};
use crate::sun::lens_flare::LensFlare;

declare_environment_keys! {
  /// The keys of a `suns.ltx` section, `CLensFlareDescriptor` (`xrEngine/xr_efflensflare.cpp`): the sun's sprite, its
  /// flares and its gradient, which a keyframe names by `sun`.
  pub enum LensFlareKey {
    /// Shadow of Chernobyl's name for the sprite switch, which OpenXRay reads before `sun`.
    Source = "source": Flag, vanilla: Conditional(Flag(false)), extended: Unread;
    SourceShader = "source_shader": Text, vanilla: Conditional(Text("")), extended: Unread;
    SourceTexture = "source_texture": Text, vanilla: Conditional(Text("")), extended: Unread;
    SourceRadius = "source_radius": Number, vanilla: Conditional(Number(0.0)), extended: Unread;
    SourceIgnoreColor = "source_ignore_color": Flag, vanilla: Conditional(Flag(false)), extended: Unread;
    /// Whether the sun's sprite is drawn.
    Sun = "sun": Flag, vanilla: Conditional(Flag(false)), extended: Required(Flag(false));
    SunShader = "sun_shader": Text, vanilla: Conditional(Text("")), extended: Conditional(Text(""));
    SunTexture = "sun_texture": Text, vanilla: Conditional(Text("")), extended: Conditional(Text(""));
    SunRadius = "sun_radius": Number, vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    SunIgnoreColor = "sun_ignore_color": Flag, vanilla: Conditional(Flag(false)), extended: Conditional(Flag(false));
    /// Whether the lens flares are drawn.
    Flares = "flares": Flag, vanilla: Required(Flag(false)), extended: Required(Flag(false));
    FlareShader = "flare_shader": Text, vanilla: Conditional(Text("")), extended: Conditional(Text(""));
    /// One flare per texture; the radius, opacity and position lists are read item by item beside it.
    FlareTextures = "flare_textures": List, vanilla: Conditional(List), extended: Conditional(List);
    FlareRadius = "flare_radius": List, vanilla: Conditional(List), extended: Conditional(List);
    FlareOpacity = "flare_opacity": List, vanilla: Conditional(List), extended: Conditional(List);
    FlarePosition = "flare_position": List, vanilla: Conditional(List), extended: Conditional(List);
    /// Whether the gradient is drawn; read as text, so only a lower-case true word turns it on.
    Gradient = "gradient": Flag, vanilla: Required(Flag(false)), extended: Required(Flag(false));
    GradientShader = "gradient_shader": Text, vanilla: Conditional(Text("")), extended: Conditional(Text(""));
    GradientTexture = "gradient_texture": Text, vanilla: Conditional(Text("")), extended: Conditional(Text(""));
    GradientRadius = "gradient_radius": Number, vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    GradientOpacity = "gradient_opacity": Number,
      vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    /// Seconds.
    BlendRiseTime = "blend_rise_time": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// Seconds.
    BlendDownTime = "blend_down_time": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
  }
}

impl LensFlareKey {
  /// The keys the sprite is drawn with, under either name.
  const SOURCE_PARTS: [Self; 4] = [
    Self::SourceShader,
    Self::SourceTexture,
    Self::SourceRadius,
    Self::SourceIgnoreColor,
  ];
  const SUN_PARTS: [Self; 4] = [Self::SunShader, Self::SunTexture, Self::SunRadius, Self::SunIgnoreColor];
  const FLARE_PARTS: [Self; 5] = [
    Self::FlareShader,
    Self::FlareTextures,
    Self::FlareRadius,
    Self::FlareOpacity,
    Self::FlarePosition,
  ];
  const GRADIENT_PARTS: [Self; 4] = [
    Self::GradientShader,
    Self::GradientTexture,
    Self::GradientRadius,
    Self::GradientOpacity,
  ];

  /// Reads one lens flare and says what its engine would refuse.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> LensFlare {
    let engine: XrayEngine = reader.get_engine();
    let flare: LensFlare = reader.read::<Self>(name, section);

    // OpenXRay reads `source` where it is written, then `sun` over it where that is written too; `sun` alone otherwise.
    if engine == XrayEngine::Vanilla && flare.has(Self::Source) {
      Self::require_parts(reader, &flare, Self::Source, &Self::SOURCE_PARTS);

      if flare.has(Self::Sun) {
        Self::require_parts(reader, &flare, Self::Sun, &Self::SUN_PARTS);
      }
    } else {
      // Required outright on Monolith, which the table already says; on OpenXRay only for want of `source`.
      if engine == XrayEngine::Vanilla {
        reader.require(&flare, Self::Sun);
      }

      Self::require_parts(reader, &flare, Self::Sun, &Self::SUN_PARTS);
    }

    Self::require_parts(reader, &flare, Self::Flares, &Self::FLARE_PARTS);
    Self::require_parts(reader, &flare, Self::Gradient, &Self::GRADIENT_PARTS);

    if flare.get_flag(Self::Flares, engine) {
      Self::judge_flare_lists(reader, &flare);
    }

    flare
  }

  /// Requires the keys a switch turns on, where it is on.
  fn require_parts(reader: &mut EnvironmentSectionReader, flare: &LensFlare, switch: Self, parts: &[Self]) {
    if flare.get_flag(switch, reader.get_engine()) {
      for part in parts {
        reader.require(flare, *part);
      }
    }
  }

  /// The engine reads a radius, an opacity and a position for each texture, a zero for any it does not find.
  fn judge_flare_lists(reader: &mut EnvironmentSectionReader, flare: &EnvironmentSection<Self>) {
    let textures: usize = flare.get_list(Self::FlareTextures).len();

    for key in [Self::FlareRadius, Self::FlareOpacity, Self::FlarePosition] {
      let count: usize = flare.get_list(key).len();

      if flare.has(key) && count < textures {
        let message: String = format!(
          "{} has {count} items in [{}] for {textures} flare textures; the engine reads the rest as zero",
          reader.describe(&flare.name),
          key.get_name()
        );

        reader.report(EnvironmentRule::Convention, &flare.name, Some(key.get_name()), message);
      }
    }
  }
}
