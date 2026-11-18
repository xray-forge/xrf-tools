use std::collections::HashMap;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::Ltx;

use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::meta::cls_id::ClsId;

/// Which class each section's objects are created as: the section's `class` key in the game's configs, or its
/// parents' (`F_entity_Create`).
///
/// Without configs, or for a section they give no class the tools read, the answer falls back to the table of Call of
/// Pripyat's sections and Clear Sky's beside it ([`ClsId::from_dialect_section`]). That table reads every section
/// vanilla Call of Pripyat's configs declare as their class's payload, and every one of Clear Sky's but two
/// multiplayer weapons.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpawnSectionClasses {
  /// Each config section with a class, lowercase as stored: its class, or the name of one the tools do not read.
  classes: HashMap<String, Result<ClsId, String>>,
}

impl SpawnSectionClasses {
  /// The configs' root file under a game data root.
  pub const SYSTEM_LTX_PATH: &'static str = "configs/system.ltx";

  /// The classes the configs declare, with includes and parents resolved.
  pub fn from_configs(ltx: &Ltx) -> Self {
    Self {
      classes: ltx
        .into_iter()
        .filter_map(|(name, section)| {
          let class: &str = section.get("class")?.trim();

          Some((
            name.to_owned(),
            ClsId::from_class_name(class).ok_or_else(|| class.to_owned()),
          ))
        })
        .collect(),
    }
  }

  /// The classes the configs of the game data at `gamedata` declare, or the table alone where it holds no
  /// `configs/system.ltx`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the file when the configs are there but do not read.
  pub fn read_from_gamedata(gamedata: &Path) -> XrfResult<Self> {
    let path = gamedata.join(Self::SYSTEM_LTX_PATH);

    if !path.is_file() {
      return Ok(Self::default());
    }

    Ltx::read_from_file_standard(&path)
      .map(|ltx| Self::from_configs(&ltx))
      .map_err(|error| XrfError::new_invalid_error(format!("The configs {}: {error}", path.display())))
  }

  /// The class objects of `section` are created as in a spawn of `dialect`'s game.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section when neither the configs nor the table give it a class the tools read.
  pub fn get_cls_id(&self, section: &str, dialect: ScriptDialect) -> XrfResult<ClsId> {
    let unread: Option<&String> = match self.classes.get(&section.to_lowercase()) {
      Some(Ok(cls_id)) => return Ok(cls_id.clone()),
      Some(Err(class)) => Some(class),
      None => None,
    };

    ClsId::from_dialect_section(section, dialect).map_err(|error| match unread {
      Some(class) => XrfError::new_parsing_error(format!(
        "ALife object section '{section}' declares class '{class}', which the spawn tools do not read"
      )),
      None => error,
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;

  use crate::data::alife::script_dialect::ScriptDialect;
  use crate::data::meta::cls_id::ClsId;
  use crate::data::meta::spawn_section_classes::SpawnSectionClasses;

  fn read(text: &str) -> XrfResult<SpawnSectionClasses> {
    Ok(SpawnSectionClasses::from_configs(
      &Ltx::read_from_str(text)?.into_inherited()?,
    ))
  }

  #[test]
  fn reads_a_mod_section_by_the_class_its_parent_declares() -> XrfResult {
    let classes: SpawnSectionClasses =
      read("[ammo_base]\nclass = AMMO_S\n\n[ammo_5.45x39_tracer]:ammo_base\ninv_name = tracer\n")?;

    assert_eq!(
      classes.get_cls_id("ammo_5.45x39_tracer", ScriptDialect::CallOfPripyat)?,
      ClsId::AmmoS
    );
    assert!(
      SpawnSectionClasses::default()
        .get_cls_id("ammo_5.45x39_tracer", ScriptDialect::CallOfPripyat)
        .is_err()
    );

    Ok(())
  }

  // Clear Sky declares its acid field an engine zone, where the table holds Call of Pripyat's scripted one.
  #[test]
  fn prefers_the_class_the_configs_declare_over_the_table() -> XrfResult {
    let classes: SpawnSectionClasses = read("[zone_field_acidic]\nclass = Z_RADIO\n")?;

    assert_eq!(
      classes.get_cls_id("zone_field_acidic", ScriptDialect::ClearSky)?,
      ClsId::ZRadio
    );
    assert_eq!(
      SpawnSectionClasses::default().get_cls_id("zone_field_acidic", ScriptDialect::ClearSky)?,
      ClsId::ZsRadio
    );

    Ok(())
  }

  #[test]
  fn falls_back_to_the_table_and_names_an_unread_class() -> XrfResult {
    let classes: SpawnSectionClasses = read("[my_drone]\nclass = DRONE_S\n\n[detector_simple]\nclass = D_FORKED\n")?;

    assert_eq!(
      classes.get_cls_id("actor", ScriptDialect::CallOfPripyat)?,
      ClsId::SActor
    );
    assert_eq!(
      classes.get_cls_id("detector_simple", ScriptDialect::CallOfPripyat)?,
      ClsId::DetSimp
    );

    let error: String = classes
      .get_cls_id("my_drone", ScriptDialect::CallOfPripyat)
      .expect_err("a class the tools do not read")
      .to_string();

    assert!(error.contains("declares class 'DRONE_S'"), "{error}");

    Ok(())
  }
}
