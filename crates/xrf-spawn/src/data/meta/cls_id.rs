#![allow(dead_code, reason = "names every engine class, including ones no tool reads yet")]

use enum_map::Enum;
use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::meta::map::{CLEAR_SKY_SECTION_TO_CLS_ID, SECTION_TO_CLS_ID};

/// todo: Add script to parse system ltx and read all the data from ltx/txt file instead.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Enum, PartialEq, Serialize, Deserialize, Eq)]
pub enum ClsId {
  AiCrow,
  AiGraph,
  AiPhant,
  AiStlS,
  AiTrdS,
  AmmoS,
  Artefact,
  CHlcpS,
  DFlare,
  DPda,
  DetAdva,
  DetElit,
  DetSimp,
  DetScie,
  EHlmet,
  EStlk,
  GF1S,
  GFake,
  GRgd5S,
  GRpg7,
  IIAttch,
  IIBolt,
  IIBttch,
  IIDoc,
  LvlChng,
  NwAttch,
  OBrkbl,
  OClmbl,
  ODstrS,
  OPhysS,
  OSearch,
  OnOffS,
  PSkelet,
  SActor,
  SExplo,
  /// `SFACTION`, Clear Sky's `se_sim_faction`.
  SFaction,
  SFood,
  SInvBox,
  SM209,
  SOG7B,
  SPda,
  SVog25,
  ScriptZn,
  ScrptArt,
  ScrptCar,
  ScrptObj,
  SmBlood,
  SmBoarW,
  SmBurer,
  SmChims,
  SmContr,
  SmDogF,
  SmDogP,
  SmDogS,
  SmFlesh,
  SmGiant,
  SmPDog,
  SmPoltr,
  SmSnork,
  SmTushk,
  SmrtCS,
  SmrtTrrn,
  SoHLamp,
  SpcRsS,
  TorchS,
  WSTMGun,
  WpAk74,
  WpAshTG,
  WpBM16,
  WpBinoc,
  WpGLaun,
  WpGroza,
  WpHPSA,
  WpKnife,
  WpLR300,
  WpPM,
  WpRG6,
  WpRPG7,
  WpSVD,
  WpSVU,
  WpScope,
  WpSilen,
  WpVAL,
  ZAcidF,
  ZAmeba,
  ZBFuzz,
  ZCFire,
  ZDead,
  ZGalant,
  ZMbald,
  ZMincer,
  ZNoGrav,
  ZRadio,
  ZRustyH,
  ZTeamBs,
  ZTorrid,
  ZsAmeba,
  ZsBFuzz,
  ZsGalan,
  ZsMBald,
  ZsMince,
  ZsNGrav,
  ZsRadio,
  ZsTorrd,
}

impl ClsId {
  /// The class a section's `class` key names, by the engine's spelling: those xrf reads a spawned object by, none for
  /// any other.
  pub fn from_class_name(class: &str) -> Option<Self> {
    match class.trim() {
      "Z_CFIRE" => Some(Self::ZCFire),
      _ => None,
    }
  }

  /// The class a section declares in Call of Pripyat's configs, which the games after it keep.
  pub fn from_section(section_name: &str) -> XrfResult<Self> {
    SECTION_TO_CLS_ID.get(section_name).cloned().ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Unknown ALife object section '{section_name}', no CLSID mapping exists"
      ))
    })
  }

  /// The class a section declares in the game whose scripts the object follows: Clear Sky's configs declare a few
  /// otherwise (`CLEAR_SKY_SECTION_TO_CLS_ID`).
  pub fn from_dialect_section(section_name: &str, dialect: ScriptDialect) -> XrfResult<Self> {
    match dialect {
      ScriptDialect::ClearSky => match CLEAR_SKY_SECTION_TO_CLS_ID.get(section_name) {
        Some(clsid) => Ok(clsid.clone()),
        None => Self::from_section(section_name),
      },
      ScriptDialect::CallOfPripyat => Self::from_section(section_name),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::ClsId;
  use crate::data::alife::script_dialect::ScriptDialect;

  #[test]
  fn reads_a_campfire_by_its_class_name() {
    assert_eq!(ClsId::from_class_name(" Z_CFIRE"), Some(ClsId::ZCFire));
    assert_eq!(ClsId::from_class_name("Z_ZONE"), None);
  }

  #[test]
  fn maps_known_section_to_clsid() {
    assert_eq!(ClsId::from_section("actor").unwrap(), ClsId::SActor);
  }

  /// Clear Sky's faction is its own class, where Call of Pripyat's section is a space restrictor.
  #[test]
  fn maps_a_section_by_the_dialect_that_declares_it() {
    assert_eq!(
      ClsId::from_dialect_section("sim_faction", ScriptDialect::ClearSky).ok(),
      Some(ClsId::SFaction)
    );
    assert_eq!(
      ClsId::from_dialect_section("sim_faction", ScriptDialect::CallOfPripyat).ok(),
      Some(ClsId::ScriptZn)
    );
    assert_eq!(
      ClsId::from_dialect_section("stalker", ScriptDialect::ClearSky).ok(),
      Some(ClsId::AiStlS)
    );
  }

  #[test]
  fn returns_error_for_unknown_section() {
    let error: XrfError = ClsId::from_section("unknown_section").unwrap_err();

    assert_eq!(
      error.to_string(),
      "Parsing error: Unknown ALife object section 'unknown_section', no CLSID mapping exists"
    );
  }
}
