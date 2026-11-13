use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_math::Vector3d;

use crate::spawn::spawn_game_level::SpawnGameLevel;
use crate::spawn::spawn_skipped_game_level::SpawnSkippedGameLevel;

/// The levels a game lists in `configs/game_levels.ltx`, in the order `[levels]` lists them, and the entries it cannot
/// take: xrAI's `read_levels` (`xr_graph_merge.cpp:397`) passes over a section that is missing or lacks its name, id
/// or offset. It reads the id as a byte, so a test level of id 10,000, as vanilla lists, becomes 16; here such an
/// entry is passed over too, and refused only where it is asked for.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnGameLevels {
  pub levels: Vec<SpawnGameLevel>,
  pub skipped: Vec<SpawnSkippedGameLevel>,
}

impl SpawnGameLevels {
  /// Where a gamedata root keeps the list.
  pub const PATH: &'static str = "configs/game_levels.ltx";
  /// The section naming each level's own.
  const LEVELS_SECTION: &'static str = "levels";

  /// Reads the list from a `game_levels.ltx`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the file when it does not read as LTX, or [`Self::from_ltx`] refuses it.
  pub fn read_from_path(path: &Path) -> XrfResult<Self> {
    Ltx::read_from_path(path)
      .and_then(|ltx| Self::from_ltx(&ltx))
      .map_err(|error| XrfError::new_invalid_error(format!("The level list {}: {error}", path.display())))
  }

  /// The levels `[levels]` names, each from its own section, names in lower case as xrAI takes them.
  ///
  /// # Errors
  ///
  /// Returns an error when `[levels]` is missing.
  pub fn from_ltx(ltx: &Ltx) -> XrfResult<Self> {
    let list: &Section = ltx.section(Self::LEVELS_SECTION).ok_or_else(|| {
      XrfError::new_parsing_error(format!("The level list has no [{}] section", Self::LEVELS_SECTION))
    })?;
    let mut levels: Self = Self {
      levels: Vec::new(),
      skipped: Vec::new(),
    };

    for (section_name, _) in list {
      let section_name: String = section_name.to_lowercase();

      match ltx.section(&section_name) {
        Some(section) => match Self::read_level(&section_name, section) {
          Ok(level) => levels.levels.push(level),
          Err(reason) => levels.skipped.push(SpawnSkippedGameLevel {
            name: Self::read(section, "name").map(|name| name.to_lowercase()),
            section: section_name,
            reason,
          }),
        },
        None => levels.skipped.push(SpawnSkippedGameLevel {
          reason: String::from("the list names it, but has no such section"),
          section: section_name,
          name: None,
        }),
      }
    }

    Ok(levels)
  }

  /// The level of directory `name`.
  ///
  /// # Errors
  ///
  /// Returns an error when the list does not name the level, or names it in an entry it passed over.
  pub fn find(&self, name: &str) -> XrfResult<&SpawnGameLevel> {
    let name: String = name.to_lowercase();

    if let Some(level) = self.levels.iter().find(|level| level.name == name) {
      return Ok(level);
    }

    Err(XrfError::new_invalid_error(
      match self.skipped.iter().find(|level| level.name.as_deref() == Some(&name)) {
        Some(skipped) => format!(
          "Level {name}'s section [{}] in {}: {}",
          skipped.section,
          Self::PATH,
          skipped.reason
        ),
        None => format!("Level {name} is not one {} lists", Self::PATH),
      },
    ))
  }

  /// A section's level, or why it is passed over.
  fn read_level(section_name: &str, section: &Section) -> Result<SpawnGameLevel, String> {
    let field = |key: &str| Self::read(section, key).ok_or_else(|| format!("it has no {key}"));
    let name: String = field("name")?.to_lowercase();
    let id: String = field("id")?;

    Ok(SpawnGameLevel {
      section: section_name.to_owned(),
      caption: Self::read(section, "caption").unwrap_or_else(|| name.clone()),
      id: id
        .parse()
        .map_err(|_| format!("its id {id} is not one of the 0 to 255 a game graph holds"))?,
      offset: field("offset")?
        .parse::<Vector3d>()
        .map_err(|error| format!("its offset does not read: {error}"))?,
      name,
    })
  }

  fn read(section: &Section, key: &str) -> Option<String> {
    section.get(key).map(|value| value.trim().trim_matches('"').to_owned())
  }
}
