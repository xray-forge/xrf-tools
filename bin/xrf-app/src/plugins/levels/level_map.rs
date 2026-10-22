//! Where a level's map lies over it: `[level_map]` of its `level.ltx`, as `CDemoRecord` and the PDA read it.

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section, read_engine_floats};
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::XrayProbe;

use crate::plugins::levels::read::read_optional_file;
use crate::plugins::levels::state::{LEVEL_CONFIG_FILE, LevelMapDescription, LevelSource};

/// The section of `level.ltx` placing the map.
const LEVEL_MAP_SECTION: &str = "level_map";

/// The map's two corners, read as `r_fvector4` reads them: `min x, min z, max x, max z`.
const BOUND_RECT_FIELD: &str = "bound_rect";

const TEXTURE_FIELD: &str = "texture";

/// The level's map, or `None` where its config is missing, places no map or cannot be read; a failure is logged.
pub fn read_level_map(source: &LevelSource, probe: &XrayProbe) -> Option<LevelMapDescription> {
  let bytes: Vec<u8> = read_optional_file(source, probe, LEVEL_CONFIG_FILE)
    .map_err(|error| log::warn!("{error}"))
    .ok()
    .flatten()?;

  decode_bytes_to_string(&bytes, new_windows1251_encoder())
    .map_err(XrfError::from)
    .and_then(|text| parse_level_map(&text))
    .map_err(|error| {
      log::warn!(
        "Failed to read '{LEVEL_CONFIG_FILE}' of level '{}': {error}",
        source.get_label()
      )
    })
    .ok()
    .flatten()
}

/// The map a level config's text places, or `None` where it has no `[level_map]`.
///
/// # Errors
///
/// Returns an error for text that is no config, or a `bound_rect` that is not four numbers spanning an area.
pub fn parse_level_map(text: &str) -> XrfResult<Option<LevelMapDescription>> {
  let ltx: Ltx = Ltx::read_from_str(text)?;
  let Some(section) = ltx.section(LEVEL_MAP_SECTION) else {
    return Ok(None);
  };
  let [min_x, min_z, max_x, max_z]: [f32; 4] = read_bound_rect(section)?;

  if !(min_x < max_x && min_z < max_z) {
    return Err(XrfError::new_invalid_error(format!(
      "[{LEVEL_MAP_SECTION}] {BOUND_RECT_FIELD} spans no area: {min_x}, {min_z}, {max_x}, {max_z}"
    )));
  }

  Ok(Some(LevelMapDescription {
    min_x,
    min_z,
    max_x,
    max_z,
    texture: section.get(TEXTURE_FIELD).map(str::to_owned),
  }))
}

fn read_bound_rect(section: &Section) -> XrfResult<[f32; 4]> {
  let value: &str = section
    .get(BOUND_RECT_FIELD)
    .ok_or_else(|| XrfError::new_invalid_error(format!("[{LEVEL_MAP_SECTION}] names no {BOUND_RECT_FIELD}")))?;
  <[f32; 4]>::try_from(read_engine_floats(value, 4)).map_err(|numbers| {
    XrfError::new_invalid_error(format!(
      "[{LEVEL_MAP_SECTION}] {BOUND_RECT_FIELD} '{value}' reads as {} numbers, not four",
      numbers.len()
    ))
  })
}
