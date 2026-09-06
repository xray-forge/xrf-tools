//! The encoder knobs both writing commands take, declared and parsed once.

use clap::{Arg, ArgMatches};
use xrf_dds::{DdsMipFilter, Quality};
use xrf_error::{XrfError, XrfResult};

/// The filter a conversion reduces with when nothing says otherwise.
pub const DEFAULT_CONVERT_MIP_FILTER: &str = "kaiser";

/// The filter a bump pair reduces with when nothing says otherwise.
///
/// Box, because that is what the SDK's generator uses: `DXTCompressBump` builds its `STextureParams` and overrides
/// only the flags, the type and the format, leaving `mip_filter` at the `kMIPFilterBox` the constructor set
/// (`xrDXT/NormalMapGen.cpp`, `editors/ECore/Editor/ETextureParams.h`).
pub const DEFAULT_BUMP_MIP_FILTER: &str = "box";

/// Quality levels the encoder offers, cheapest first.
const QUALITY_NAMES: [&str; 3] = ["fast", "normal", "slow"];

/// The kernel argument, defaulted by whichever command declares it.
pub fn new_mip_filter_argument(default: &'static str) -> Arg {
  Arg::new("mip-filter")
    .help("Kernel the mip chain is reduced with, from the X-Ray converter's own family")
    .long("mip-filter")
    .default_value(default)
    .value_parser(get_filter_names())
}

/// The quality argument, which trades encoding time for fidelity.
pub fn new_quality_argument() -> Arg {
  Arg::new("quality")
    .help("How hard the encoder works; `slow` costs seconds on BC7 and pennies on the rest")
    .long("quality")
    .default_value("slow")
    .value_parser(QUALITY_NAMES)
}

/// The kernel a run asked for.
pub fn get_mip_filter(matches: &ArgMatches) -> XrfResult<DdsMipFilter> {
  let name: &str = matches
    .get_one::<String>("mip-filter")
    .map(String::as_str)
    .unwrap_or_default();

  DdsMipFilter::from_label(name).ok_or_else(|| XrfError::new_invalid_error(format!("Unexpected mip filter '{name}'")))
}

/// The quality a run asked for.
pub fn get_quality(matches: &ArgMatches) -> XrfResult<Quality> {
  match matches.get_one::<String>("quality").map(String::as_str) {
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
fn get_filter_names() -> Vec<String> {
  DdsMipFilter::NAMED
    .iter()
    .map(|filter| filter.label().to_lowercase())
    .collect()
}
