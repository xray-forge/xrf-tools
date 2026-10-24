use clap::{Arg, ArgMatches, Command};
use xrf_dds::{DdsMipFilter, Quality};
use xrf_error::{XrfError, XrfResult};

/// `--mip-filter` and `--quality`: the encoder knobs both writing commands take, declared and parsed once.
pub struct DdsEncodeArguments;

impl DdsEncodeArguments {
  const MIP_FILTER: &'static str = "mip-filter";
  const QUALITY: &'static str = "quality";

  /// The filter a conversion reduces with when nothing says otherwise.
  pub const CONVERT_MIP_FILTER: DdsMipFilter = DdsMipFilter::Kaiser;

  /// The filter a bump pair reduces with when nothing says otherwise.
  ///
  /// Box, because that is what the SDK's generator uses: `DXTCompressBump` builds its `STextureParams` and overrides
  /// only the flags, the type and the format, leaving `mip_filter` at the `kMIPFilterBox` the constructor set
  /// (`xrDXT/NormalMapGen.cpp`, `editors/ECore/Editor/ETextureParams.h`).
  pub const BUMP_MIP_FILTER: DdsMipFilter = DdsMipFilter::Box;

  /// Quality levels the encoder offers, cheapest first.
  const QUALITY_NAMES: [&'static str; 3] = ["fast", "normal", "slow"];

  /// Declares both on `command`, the mip chain reduced with `default_filter` unless the run names another.
  pub fn declare(command: Command, default_filter: DdsMipFilter) -> Command {
    command
      .arg(
        Arg::new(Self::MIP_FILTER)
          .help("Kernel the mip chain is reduced with, from the X-Ray converter's own family")
          .long(Self::MIP_FILTER)
          .default_value(default_filter.label().to_lowercase())
          .value_parser(Self::list_filter_names()),
      )
      .arg(
        Arg::new(Self::QUALITY)
          .help("How hard the encoder works; `slow` costs seconds on BC7 and pennies on the rest")
          .long(Self::QUALITY)
          .default_value("slow")
          .value_parser(Self::QUALITY_NAMES),
      )
  }

  /// The kernel a run asked for.
  ///
  /// # Errors
  ///
  /// Returns an error when the name is no filter of the family.
  pub fn mip_filter_of(matches: &ArgMatches) -> XrfResult<DdsMipFilter> {
    let name: &str = matches
      .get_one::<String>(Self::MIP_FILTER)
      .map(String::as_str)
      .unwrap_or_default();

    DdsMipFilter::from_label(name).ok_or_else(|| XrfError::new_invalid_error(format!("Unexpected mip filter '{name}'")))
  }

  /// The quality a run asked for.
  ///
  /// # Errors
  ///
  /// Returns an error when the name is no quality the encoder offers.
  pub fn quality_of(matches: &ArgMatches) -> XrfResult<Quality> {
    match matches.get_one::<String>(Self::QUALITY).map(String::as_str) {
      Some("fast") => Ok(Quality::Fast),
      Some("normal") => Ok(Quality::Normal),
      Some("slow") => Ok(Quality::Slow),
      other => Err(XrfError::new_invalid_error(format!(
        "Unexpected encoder quality '{}'",
        other.unwrap_or_default()
      ))),
    }
  }

  /// The filter names accepted, taken from the family itself so the two cannot drift.
  fn list_filter_names() -> Vec<String> {
    DdsMipFilter::NAMED
      .iter()
      .map(|filter| filter.label().to_lowercase())
      .collect()
  }
}
