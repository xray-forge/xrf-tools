use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section, read_engine_floats};
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::core::session::{SessionPart, SessionParts, SessionSnapshot};
use crate::plugins::levels::state::{LEVEL_CONFIG_FILE, LevelMapDescription, LevelSource, SelectedLevel};

/// What a change event calls where a level's map lies when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_MAP_PART: &str = "levelMap";

/// Where the level's map lies, read from its `level.ltx` alone, so an edit of the config moves the map and not the
/// level.
pub(crate) struct LevelMapPart;

impl SessionPart for LevelMapPart {
  type Value = Option<LevelMapDescription>;

  const NAME: &'static str = LEVEL_MAP_PART;

  // Read from one small config, which a thread of its own would only delay.
  const IS_POOLED: bool = false;
}

impl LevelMapPart {
  /// The section of `level.ltx` placing the map.
  const SECTION: &'static str = "level_map";

  /// The map's two corners, read as `r_fvector4` reads them: `min x, min z, max x, max z`.
  const BOUND_RECT_FIELD: &'static str = "bound_rect";

  const TEXTURE_FIELD: &'static str = "texture";

  /// The store with `map` kept as the level's map part, read from `inputs` as the level opened.
  pub fn with(parts: SessionParts, map: Option<LevelMapDescription>, inputs: XrayInputRecord) -> SessionParts {
    parts.with::<Self>(map, inputs)
  }

  /// Where the open level's map lies, read the first time anything asks and kept; `None` where its config places none.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Option<LevelMapDescription> {
    current
      .parts
      .get_or_record::<Self>(search, |search| Self::read(&current.source, search))
  }

  /// The level's map, or `None` where its config is missing, places no map or cannot be read; a failure is logged.
  pub fn read(source: &LevelSource, search: &XraySearch) -> Option<LevelMapDescription> {
    let bytes: Vec<u8> = source
      .read_optional_file(search, LEVEL_CONFIG_FILE)
      .map_err(|error| log::warn!("{error}"))
      .ok()
      .flatten()?;

    decode_bytes_to_string(&bytes, new_windows1251_encoder())
      .map_err(XrfError::from)
      .and_then(|text| Self::parse(&text))
      .map_err(|error| log::warn!("{}", source.describe_read_failure(LEVEL_CONFIG_FILE, &error)))
      .ok()
      .flatten()
  }

  /// The map a level config's text places, or `None` where it has no `[level_map]`.
  ///
  /// # Errors
  ///
  /// Returns an error for text that is no config, or a `bound_rect` that is not four numbers spanning an area.
  pub fn parse(text: &str) -> XrfResult<Option<LevelMapDescription>> {
    let ltx: Ltx = Ltx::read_from_str(text)?;
    let Some(section) = ltx.section(Self::SECTION) else {
      return Ok(None);
    };
    let [min_x, min_z, max_x, max_z]: [f32; 4] = Self::read_bound_rect(section)?;

    if !(min_x < max_x && min_z < max_z) {
      return Err(XrfError::new_invalid_error(format!(
        "[{}] {} spans no area: {min_x}, {min_z}, {max_x}, {max_z}",
        Self::SECTION,
        Self::BOUND_RECT_FIELD
      )));
    }

    Ok(Some(LevelMapDescription {
      min_x,
      min_z,
      max_x,
      max_z,
      texture: section.get(Self::TEXTURE_FIELD).map(str::to_owned),
    }))
  }

  fn read_bound_rect(section: &Section) -> XrfResult<[f32; 4]> {
    let value: &str = section
      .get(Self::BOUND_RECT_FIELD)
      .ok_or_else(|| XrfError::new_invalid_error(format!("[{}] names no {}", Self::SECTION, Self::BOUND_RECT_FIELD)))?;

    <[f32; 4]>::try_from(read_engine_floats(value, 4)).map_err(|numbers| {
      XrfError::new_invalid_error(format!(
        "[{}] {} '{value}' reads as {} numbers, not four",
        Self::SECTION,
        Self::BOUND_RECT_FIELD,
        numbers.len()
      ))
    })
  }
}
