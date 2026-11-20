#![allow(dead_code, reason = "names every engine class, including ones no tool reads yet")]

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::meta::map::{CLEAR_SKY_SECTION_TO_CLS_ID, SECTION_TO_CLS_ID};

/// The class a spawned object is created as, named by the configs' `class` key; `SpawnSectionClasses` says which a
/// section is.
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
  /// The class's name as the configs' `class` key spells it: `xray::make_clsid`'s eight characters, unpadded.
  pub fn get_class_name(&self) -> &'static str {
    match self {
      Self::AiCrow => "AI_CROW",
      Self::AiGraph => "AI_GRAPH",
      Self::AiPhant => "AI_PHANT",
      Self::AiStlS => "AI_STL_S",
      Self::AiTrdS => "AI_TRD_S",
      Self::AmmoS => "AMMO_S",
      Self::Artefact => "ARTEFACT",
      Self::CHlcpS => "C_HLCP_S",
      Self::DFlare => "D_FLARE",
      Self::DPda => "D_PDA",
      Self::DetAdva => "DET_ADVA",
      Self::DetElit => "DET_ELIT",
      Self::DetSimp => "DET_SIMP",
      Self::DetScie => "DET_SCIE",
      Self::EHlmet => "E_HLMET",
      Self::EStlk => "E_STLK",
      Self::GF1S => "G_F1_S",
      Self::GFake => "G_FAKE",
      Self::GRgd5S => "G_RGD5_S",
      Self::GRpg7 => "G_RPG7",
      Self::IIAttch => "II_ATTCH",
      Self::IIBolt => "II_BOLT",
      Self::IIBttch => "II_BTTCH",
      Self::IIDoc => "II_DOC",
      Self::LvlChng => "LVL_CHNG",
      Self::NwAttch => "NW_ATTCH",
      Self::OBrkbl => "O_BRKBL",
      Self::OClmbl => "O_CLMBL",
      Self::ODstrS => "O_DSTR_S",
      Self::OPhysS => "O_PHYS_S",
      Self::OSearch => "O_SEARCH",
      Self::OnOffS => "ON_OFF_S",
      Self::PSkelet => "P_SKELET",
      Self::SActor => "S_ACTOR",
      Self::SExplo => "S_EXPLO",
      Self::SFaction => "SFACTION",
      Self::SFood => "S_FOOD",
      Self::SInvBox => "S_INVBOX",
      Self::SM209 => "S_M209",
      Self::SOG7B => "S_OG7B",
      Self::SPda => "S_PDA",
      Self::SVog25 => "S_VOG25",
      Self::ScriptZn => "SCRIPTZN",
      Self::ScrptArt => "SCRPTART",
      Self::ScrptCar => "SCRPTCAR",
      Self::ScrptObj => "SCRPTOBJ",
      Self::SmBlood => "SM_BLOOD",
      Self::SmBoarW => "SM_BOARW",
      Self::SmBurer => "SM_BURER",
      Self::SmChims => "SM_CHIMS",
      Self::SmContr => "SM_CONTR",
      Self::SmDogF => "SM_DOG_F",
      Self::SmDogP => "SM_DOG_P",
      Self::SmDogS => "SM_DOG_S",
      Self::SmFlesh => "SM_FLESH",
      Self::SmGiant => "SM_GIANT",
      Self::SmPDog => "SM_P_DOG",
      Self::SmPoltr => "SM_POLTR",
      Self::SmSnork => "SM_SNORK",
      Self::SmTushk => "SM_TUSHK",
      Self::SmrtCS => "SMRT_C_S",
      Self::SmrtTrrn => "SMRTTRRN",
      Self::SoHLamp => "SO_HLAMP",
      Self::SpcRsS => "SPC_RS_S",
      Self::TorchS => "TORCH_S",
      Self::WSTMGun => "W_STMGUN",
      Self::WpAk74 => "WP_AK74",
      Self::WpAshTG => "WP_ASHTG",
      Self::WpBM16 => "WP_BM16",
      Self::WpBinoc => "WP_BINOC",
      Self::WpGLaun => "WP_GLAUN",
      Self::WpGroza => "WP_GROZA",
      Self::WpHPSA => "WP_HPSA",
      Self::WpKnife => "WP_KNIFE",
      Self::WpLR300 => "WP_LR300",
      Self::WpPM => "WP_PM",
      Self::WpRG6 => "WP_RG6",
      Self::WpRPG7 => "WP_RPG7",
      Self::WpSVD => "WP_SVD",
      Self::WpSVU => "WP_SVU",
      Self::WpScope => "WP_SCOPE",
      Self::WpSilen => "WP_SILEN",
      Self::WpVAL => "WP_VAL",
      Self::ZAcidF => "Z_ACIDF",
      Self::ZAmeba => "Z_AMEBA",
      Self::ZBFuzz => "Z_BFUZZ",
      Self::ZCFire => "Z_CFIRE",
      Self::ZDead => "Z_DEAD",
      Self::ZGalant => "Z_GALANT",
      Self::ZMbald => "Z_MBALD",
      Self::ZMincer => "Z_MINCER",
      Self::ZNoGrav => "Z_NOGRAV",
      Self::ZRadio => "Z_RADIO",
      Self::ZRustyH => "Z_RUSTYH",
      Self::ZTeamBs => "Z_TEAMBS",
      Self::ZTorrid => "Z_TORRID",
      Self::ZsAmeba => "ZS_AMEBA",
      Self::ZsBFuzz => "ZS_BFUZZ",
      Self::ZsGalan => "ZS_GALAN",
      Self::ZsMBald => "ZS_MBALD",
      Self::ZsMince => "ZS_MINCE",
      Self::ZsNGrav => "ZS_NGRAV",
      Self::ZsRadio => "ZS_RADIO",
      Self::ZsTorrd => "ZS_TORRD",
    }
  }

  /// The class a `class` key names: each class by its own name ([`Self::get_class_name`]), plus the engine, multiplayer
  /// and Clear Sky spellings of the class with the same payload. `None` for a class no spawned object is read as.
  pub fn from_class_name(class: &str) -> Option<Self> {
    let class: &str = match class.trim() {
      "AMMO" => "AMMO_S",
      "A_M209" => "S_M209",
      "A_OG7B" => "S_OG7B",
      "A_VOG25" => "S_VOG25",
      "D_ADVANC" => "DET_ADVA",
      "D_ELITE" => "DET_ELIT",
      "D_SIMDET" => "DET_SIMP",
      "EQU_EXO" | "EQU_MLTR" | "EQU_SCIE" => "E_STLK",
      "G_F1" => "G_F1_S",
      "G_RGD5" => "G_RGD5_S",
      "II_EXPLO" => "S_EXPLO",
      "S_MEDKI" | "II_ANTIR" | "II_BANDG" | "II_BOTTL" | "II_FOOD" | "II_MEDKI" => "S_FOOD",
      "O_HLAMP" => "SO_HLAMP",
      "O_INVBOX" => "S_INVBOX",
      "P_DSTRBL" => "O_DSTR_S",
      "W_AK74" => "WP_AK74",
      "WP_SHOTG" | "W_SHOTGN" => "WP_ASHTG",
      "W_BM16" => "WP_BM16",
      "W_BINOC" => "WP_BINOC",
      "W_GLAUNC" => "WP_GLAUN",
      "W_GROZA" => "WP_GROZA",
      "W_HPSA" => "WP_HPSA",
      "W_KNIFE" => "WP_KNIFE",
      "WP_USP45" | "W_USP45" => "WP_LR300",
      "W_PM" | "W_FORT" => "WP_PM",
      "W_RG6" => "WP_RG6",
      "W_RPG7" => "WP_RPG7",
      "W_SVD" => "WP_SVD",
      "W_SCOPE" => "WP_SCOPE",
      "W_SILENC" => "WP_SILEN",
      "W_VAL" => "WP_VAL",
      class => class,
    };

    Self::all().find(|cls_id| cls_id.get_class_name() == class)
  }

  /// Every class, in declaration order.
  pub fn all() -> impl Iterator<Item = Self> {
    EnumMap::<Self, ()>::default().into_iter().map(|(cls_id, ())| cls_id)
  }

  /// The class a section declares in Call of Pripyat's configs, which the games after it keep.
  pub fn from_section(section_name: &str) -> XrfResult<Self> {
    SECTION_TO_CLS_ID.get(section_name).cloned().ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Unknown ALife object section '{section_name}', no CLSID mapping exists"
      ))
    })
  }

  /// The class a section declares under `dialect`: Clear Sky's configs declare a few otherwise, and some only there
  /// (`CLEAR_SKY_SECTION_TO_CLS_ID`).
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

  // A class added without a name would go unread from the configs.
  #[test]
  fn reads_every_class_by_its_own_name() {
    for cls_id in ClsId::all() {
      assert_eq!(
        ClsId::from_class_name(cls_id.get_class_name()),
        Some(cls_id.clone()),
        "{cls_id:?}"
      );
    }
  }

  #[test]
  fn reads_an_engine_spelling_as_its_class() {
    assert_eq!(ClsId::from_class_name("W_AK74"), Some(ClsId::WpAk74));
    assert_eq!(ClsId::from_class_name("II_MEDKI"), Some(ClsId::SFood));
    assert_eq!(ClsId::from_class_name("D_SIMDET"), Some(ClsId::DetSimp));
  }

  #[test]
  fn maps_known_section_to_clsid() {
    assert_eq!(ClsId::from_section("actor").unwrap(), ClsId::SActor);
  }

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
