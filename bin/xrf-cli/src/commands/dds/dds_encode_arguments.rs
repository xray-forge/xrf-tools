use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Arg, ArgMatches, Command};
use xrf_dds::{DdsMipFilter, Quality};

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

  /// Quality levels the encoder offers, cheapest first, each by the name the command line spells it.
  const QUALITIES: [(&'static str, Quality); 3] = [
    ("fast", Quality::Fast),
    ("normal", Quality::Normal),
    ("slow", Quality::Slow),
  ];

  /// Declares both on `command`, the mip chain reduced with `default_filter` unless the run names another.
  ///
  /// Each value is parsed into its type by clap, so a name outside the family is a usage error and the readers below
  /// cannot meet one.
  pub fn declare(command: Command, default_filter: DdsMipFilter) -> Command {
    command
      .arg(
        Arg::new(Self::MIP_FILTER)
          .help("Kernel the mip chain is reduced with, from the X-Ray converter's own family")
          .long(Self::MIP_FILTER)
          .default_value(default_filter.label().to_lowercase())
          .value_parser(
            PossibleValuesParser::new(Self::list_filter_names()).map(|name: String| {
              DdsMipFilter::from_label(&name).expect("Expected every possible value to name a filter of the family")
            }),
          ),
      )
      .arg(
        Arg::new(Self::QUALITY)
          .help("How hard the encoder works; `slow` costs seconds on BC7 and pennies on the rest")
          .long(Self::QUALITY)
          .default_value("slow")
          .value_parser(
            PossibleValuesParser::new(Self::QUALITIES.map(|(name, _)| name)).map(|name: String| {
              Self::QUALITIES
                .into_iter()
                .find_map(|(candidate, quality)| (candidate == name).then_some(quality))
                .expect("Expected every possible value to name a quality")
            }),
          ),
      )
  }

  /// The kernel a run asked for, or the default it was declared with.
  pub fn mip_filter_of(matches: &ArgMatches) -> DdsMipFilter {
    *matches
      .get_one::<DdsMipFilter>(Self::MIP_FILTER)
      .expect("Expected the mip filter to have a default")
  }

  /// The quality a run asked for, or the default.
  pub fn quality_of(matches: &ArgMatches) -> Quality {
    *matches
      .get_one::<Quality>(Self::QUALITY)
      .expect("Expected the quality to have a default")
  }

  /// The filter names accepted, taken from the family itself so the two cannot drift.
  fn list_filter_names() -> Vec<String> {
    DdsMipFilter::NAMED
      .iter()
      .map(|filter| filter.label().to_lowercase())
      .collect()
  }
}
