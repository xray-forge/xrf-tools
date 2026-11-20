use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Arg, ArgAction, ArgGroup, ArgMatches, Command, value_parser};
use xrf_error::{XrfError, XrfResult};
use xrf_thm::{
  ThmBumpMode, ThmFormat, ThmMaterial, ThmMipFilter, ThmTextureEdit, ThmTextureFlag, ThmTextureFlags, ThmTextureType,
};

/// The descriptor fields `thm write` and `thm set-field` take, one named flag each, declared and read once.
///
/// Every choice is declared to clap, so a misspelt value is a usage error and the readers below cannot meet one.
pub(crate) struct ThmFieldArguments;

impl ThmFieldArguments {
  /// The group every field flag belongs to, which `thm set-field` requires one member of.
  pub(crate) const GROUP: &'static str = "fields";

  const TEXTURE_TYPE: &'static str = "texture-type";
  const FORMAT: &'static str = "format";
  const MIP_FILTER: &'static str = "mip-filter";
  const BUMP_MODE: &'static str = "bump-mode";
  const BUMP_NAME: &'static str = "bump-name";
  const DETAIL_NAME: &'static str = "detail-name";
  const DETAIL_SCALE: &'static str = "detail-scale";
  const MATERIAL: &'static str = "material";
  const MATERIAL_WEIGHT: &'static str = "material-weight";
  const BORDER_COLOR: &'static str = "border-color";
  const FADE_COLOR: &'static str = "fade-color";
  const FADE_AMOUNT: &'static str = "fade-amount";
  const FADE_DELAY: &'static str = "fade-delay";
  const EXT_NORMAL_MAP: &'static str = "ext-normal-map";
  const VIRTUAL_HEIGHT: &'static str = "virtual-height";
  const SET_FLAG: &'static str = "set-flag";
  const CLEAR_FLAG: &'static str = "clear-flag";

  /// The types the SDK names, each by the name the command line spells it.
  const TEXTURE_TYPES: [(&'static str, ThmTextureType); 5] = [
    ("image", ThmTextureType::Image),
    ("cube-map", ThmTextureType::CubeMap),
    ("bump-map", ThmTextureType::BumpMap),
    ("normal-map", ThmTextureType::NormalMap),
    ("terrain", ThmTextureType::Terrain),
  ];

  /// The formats the SDK names, in its declaration order.
  const FORMATS: [(&'static str, ThmFormat); 14] = [
    ("dxt1", ThmFormat::Dxt1),
    ("dxt1-alpha", ThmFormat::Dxt1Alpha),
    ("dxt3", ThmFormat::Dxt3),
    ("dxt5", ThmFormat::Dxt5),
    ("rgba4444", ThmFormat::Rgba4444),
    ("rgba1555", ThmFormat::Rgba1555),
    ("rgb565", ThmFormat::Rgb565),
    ("rgb", ThmFormat::Rgb),
    ("rgba", ThmFormat::Rgba),
    ("nvhs", ThmFormat::Nvhs),
    ("nvhu", ThmFormat::Nvhu),
    ("a8", ThmFormat::A8),
    ("l8", ThmFormat::L8),
    ("a8l8", ThmFormat::A8L8),
  ];

  /// The bump modes an editor offers.
  const BUMP_MODES: [(&'static str, ThmBumpMode); 3] = [
    ("none", ThmBumpMode::None),
    ("use", ThmBumpMode::Use),
    ("use-parallax", ThmBumpMode::UseParallax),
  ];

  /// The shading pairs the SDK names.
  const MATERIALS: [(&'static str, ThmMaterial); 4] = [
    ("oren-nayar-blin", ThmMaterial::OrenNayarBlin),
    ("blin-phong", ThmMaterial::BlinPhong),
    ("phong-metal", ThmMaterial::PhongMetal),
    ("metal-oren-nayar", ThmMaterial::MetalOrenNayar),
  ];

  /// Declares every field flag on `command`, in the group [`Self::GROUP`], none of them required.
  pub(crate) fn declare(command: Command) -> Command {
    command
      .arg(Self::choice(
        Self::TEXTURE_TYPE,
        "Texture type, the gate the engine reads first",
        &Self::TEXTURE_TYPES,
      ))
      .arg(Self::choice(
        Self::FORMAT,
        "Format the converter builds the texture in",
        &Self::FORMATS,
      ))
      .arg(
        Self::named(
          Self::MIP_FILTER,
          "Filter the converter reduces each mip level with; 'advanced' selects the SDK's own chain",
        )
        .value_parser(
          PossibleValuesParser::new(ThmMipFilter::NAMED.map(|filter| filter.label().to_lowercase())).map(
            |name: String| {
              ThmMipFilter::NAMED
                .into_iter()
                .find(|filter| filter.label().eq_ignore_ascii_case(&name))
                .expect("Expected every possible value to name a filter")
            },
          ),
        ),
      )
      .arg(Self::choice(
        Self::BUMP_MODE,
        "Whether the renderer binds the bump pair",
        &Self::BUMP_MODES,
      ))
      .arg(Self::text(
        Self::BUMP_NAME,
        "Bump texture, engine style without extension, for example 'wpn\\wpn_pm\\wpn_pm_bump'",
      ))
      .arg(Self::text(
        Self::DETAIL_NAME,
        "Detail texture, engine style without extension; empty for none",
      ))
      .arg(
        Self::named(
          Self::DETAIL_SCALE,
          "How many times the detail texture repeats over the texture",
        )
        .value_parser(value_parser!(f32)),
      )
      .arg(Self::choice(
        Self::MATERIAL,
        "Shading pair the renderer blends between",
        &Self::MATERIALS,
      ))
      .arg(
        Self::named(Self::MATERIAL_WEIGHT, "Where between the two shadings the texture sits")
          .value_parser(value_parser!(f32)),
      )
      .arg(Self::color(Self::BORDER_COLOR, "Border colour, AARRGGBB in hex"))
      .arg(Self::color(
        Self::FADE_COLOR,
        "Colour the mip chain fades towards, AARRGGBB in hex; its alpha is the alpha fade's target",
      ))
      .arg(
        Self::named(
          Self::FADE_AMOUNT,
          "Percent of the way each faded mip level moves towards the fade colour",
        )
        .value_parser(value_parser!(u32).range(0..=100)),
      )
      .arg(Self::named(Self::FADE_DELAY, "First mip level the fade reaches").value_parser(value_parser!(u8)))
      .arg(Self::text(
        Self::EXT_NORMAL_MAP,
        "Normal map the bump generator takes instead of deriving one, engine style; empty for none",
      ))
      .arg(
        Self::named(
          Self::VIRTUAL_HEIGHT,
          "Relief height the bump generator reads, and nothing at runtime",
        )
        .value_parser(value_parser!(f32)),
      )
      .arg(Self::flags(
        Self::SET_FLAG,
        "Flag to set, by its SDK name; repeat or separate with commas",
      ))
      .arg(Self::flags(
        Self::CLEAR_FLAG,
        "Flag to clear, by its SDK name; repeat or separate with commas",
      ))
      .group(
        ArgGroup::new(Self::GROUP)
          .args([
            Self::TEXTURE_TYPE,
            Self::FORMAT,
            Self::MIP_FILTER,
            Self::BUMP_MODE,
            Self::BUMP_NAME,
            Self::DETAIL_NAME,
            Self::DETAIL_SCALE,
            Self::MATERIAL,
            Self::MATERIAL_WEIGHT,
            Self::BORDER_COLOR,
            Self::FADE_COLOR,
            Self::FADE_AMOUNT,
            Self::FADE_DELAY,
            Self::EXT_NORMAL_MAP,
            Self::VIRTUAL_HEIGHT,
            Self::SET_FLAG,
            Self::CLEAR_FLAG,
          ])
          .multiple(true),
      )
  }

  /// The fields a run names.
  ///
  /// # Errors
  ///
  /// Returns an error when the run both sets and clears one flag, which says nothing about which it meant.
  pub(crate) fn of(matches: &ArgMatches) -> XrfResult<ThmTextureEdit> {
    let set_flags: Vec<ThmTextureFlag> = Self::flags_of(matches, Self::SET_FLAG);
    let cleared_flags: Vec<ThmTextureFlag> = Self::flags_of(matches, Self::CLEAR_FLAG);

    if let Some(flag) = set_flags.iter().find(|flag| cleared_flags.contains(flag)) {
      return Err(XrfError::new_invalid_error(format!(
        "Flag '{}' is both set and cleared",
        flag.label()
      )));
    }

    Ok(ThmTextureEdit {
      texture_type: matches.get_one::<ThmTextureType>(Self::TEXTURE_TYPE).copied(),
      format: matches.get_one::<ThmFormat>(Self::FORMAT).copied(),
      mip_filter: matches.get_one::<ThmMipFilter>(Self::MIP_FILTER).copied(),
      bump_mode: matches.get_one::<ThmBumpMode>(Self::BUMP_MODE).copied(),
      bump_name: matches.get_one::<String>(Self::BUMP_NAME).cloned(),
      detail_name: matches.get_one::<String>(Self::DETAIL_NAME).cloned(),
      detail_scale: matches.get_one::<f32>(Self::DETAIL_SCALE).copied(),
      material: matches.get_one::<ThmMaterial>(Self::MATERIAL).copied(),
      material_weight: matches.get_one::<f32>(Self::MATERIAL_WEIGHT).copied(),
      border_color: matches.get_one::<u32>(Self::BORDER_COLOR).copied(),
      fade_color: matches.get_one::<u32>(Self::FADE_COLOR).copied(),
      fade_amount: matches.get_one::<u32>(Self::FADE_AMOUNT).copied(),
      fade_delay: matches.get_one::<u8>(Self::FADE_DELAY).copied(),
      ext_normal_map_name: matches.get_one::<String>(Self::EXT_NORMAL_MAP).cloned(),
      virtual_height: matches.get_one::<f32>(Self::VIRTUAL_HEIGHT).copied(),
      set_flags: Self::mask_of(&set_flags),
      cleared_flags: Self::mask_of(&cleared_flags),
      ..ThmTextureEdit::default()
    })
  }

  /// The flag word with exactly `flags` set.
  fn mask_of(flags: &[ThmTextureFlag]) -> ThmTextureFlags {
    flags
      .iter()
      .fold(ThmTextureFlags::none(), |mask, flag| mask.with(*flag, true))
  }

  /// A flag taking one of `table`'s names, parsed into its value.
  fn choice<T: Copy + Send + Sync + 'static>(
    id: &'static str,
    help: &'static str,
    table: &'static [(&'static str, T)],
  ) -> Arg {
    Self::named(id, help).value_parser(PossibleValuesParser::new(table.iter().map(|(name, _)| *name)).map(
      move |name: String| {
        table
          .iter()
          .find_map(|(candidate, value)| (*candidate == name).then_some(*value))
          .expect("Expected every possible value to name a member of the table")
      },
    ))
  }

  /// A long flag of its own name.
  fn named(id: &'static str, help: &'static str) -> Arg {
    Arg::new(id).help(help).long(id)
  }

  fn text(id: &'static str, help: &'static str) -> Arg {
    Self::named(id, help).value_parser(value_parser!(String))
  }

  fn color(id: &'static str, help: &'static str) -> Arg {
    Self::named(id, help).value_parser(Self::parse_color)
  }

  fn flags(id: &'static str, help: &'static str) -> Arg {
    Self::named(id, help)
      .action(ArgAction::Append)
      .value_delimiter(',')
      .value_parser(
        PossibleValuesParser::new(ThmTextureFlag::NAMED.map(ThmTextureFlag::label)).map(|name: String| {
          ThmTextureFlag::NAMED
            .into_iter()
            .find(|flag| flag.label() == name)
            .expect("Expected every possible value to name a flag")
        }),
      )
  }

  fn flags_of(matches: &ArgMatches, id: &str) -> Vec<ThmTextureFlag> {
    matches
      .get_many::<ThmTextureFlag>(id)
      .map(|flags| flags.copied().collect())
      .unwrap_or_default()
  }

  /// A colour as a descriptor stores it, `AARRGGBB`, written in hex with or without a `0x` or `#` before it.
  fn parse_color(text: &str) -> Result<u32, String> {
    let digits: &str = text
      .strip_prefix("0x")
      .or_else(|| text.strip_prefix("0X"))
      .or_else(|| text.strip_prefix('#'))
      .unwrap_or(text);

    if digits.is_empty() || digits.len() > 8 {
      return Err(format!(
        "'{text}' is not a colour of at most eight hex digits, AARRGGBB"
      ));
    }

    u32::from_str_radix(digits, 16)
      .map_err(|_| format!("'{text}' is not a colour of at most eight hex digits, AARRGGBB"))
  }
}

#[cfg(test)]
mod tests {
  use xrf_thm::{ThmBumpMode, ThmFormat, ThmMaterial, ThmTextureType};

  use super::ThmFieldArguments;

  #[test]
  fn names_every_member_the_sdk_names() {
    for format in ThmFormat::NAMED {
      assert!(
        ThmFieldArguments::FORMATS.iter().any(|(_, value)| *value == format),
        "{format:?}"
      );
    }

    for texture_type in ThmTextureType::NAMED {
      assert!(
        ThmFieldArguments::TEXTURE_TYPES
          .iter()
          .any(|(_, value)| *value == texture_type)
      );
    }

    for mode in ThmBumpMode::NAMED {
      assert!(ThmFieldArguments::BUMP_MODES.iter().any(|(_, value)| *value == mode));
    }

    for material in ThmMaterial::NAMED {
      assert!(ThmFieldArguments::MATERIALS.iter().any(|(_, value)| *value == material));
    }
  }

  #[test]
  fn reads_a_colour_in_hex_with_or_without_a_prefix() {
    assert_eq!(ThmFieldArguments::parse_color("7F7F7F7F"), Ok(0x7F7F_7F7F));
    assert_eq!(ThmFieldArguments::parse_color("0x00ff8000"), Ok(0x00FF_8000));
    assert_eq!(ThmFieldArguments::parse_color("#ff"), Ok(0xFF));
    assert!(ThmFieldArguments::parse_color("0x").is_err());
    assert!(ThmFieldArguments::parse_color("123456789").is_err());
    assert!(ThmFieldArguments::parse_color("red").is_err());
  }
}
