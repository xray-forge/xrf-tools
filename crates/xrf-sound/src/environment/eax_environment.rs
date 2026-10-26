use serde::{Deserialize, Serialize};

/// The EAX 2 room a sound environment was authored from, `EAX_ENVIRONMENT_*` (`sdk/include/eax/eax.h`), numbered as
/// EAX numbers them.
///
/// Only authoring reads it: the SDK's environment editor filled every reverb parameter from the room chosen by asking
/// the sound card's EAX driver, which held the preset table, while the engine plays the twelve stored parameters and
/// hands EAX its default room (`CSoundRender_EffectsA_EAX::set_listener`). xray-16 has that editor call commented out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum EaxEnvironment {
  Generic,
  PaddedCell,
  Room,
  Bathroom,
  LivingRoom,
  StoneRoom,
  Auditorium,
  ConcertHall,
  Cave,
  Arena,
  Hangar,
  CarpetedHallway,
  Hallway,
  StoneCorridor,
  Alley,
  Forest,
  City,
  Mountains,
  Quarry,
  Plain,
  ParkingLot,
  SewerPipe,
  Underwater,
  Drugged,
  Dizzy,
  Psychotic,
  /// A number EAX has no room for, kept as it was read so a library written back says what it said.
  Unknown(u32),
}

impl EaxEnvironment {
  /// Every room EAX names, in its numbering.
  pub const NAMED: [Self; 26] = [
    Self::Generic,
    Self::PaddedCell,
    Self::Room,
    Self::Bathroom,
    Self::LivingRoom,
    Self::StoneRoom,
    Self::Auditorium,
    Self::ConcertHall,
    Self::Cave,
    Self::Arena,
    Self::Hangar,
    Self::CarpetedHallway,
    Self::Hallway,
    Self::StoneCorridor,
    Self::Alley,
    Self::Forest,
    Self::City,
    Self::Mountains,
    Self::Quarry,
    Self::Plain,
    Self::ParkingLot,
    Self::SewerPipe,
    Self::Underwater,
    Self::Drugged,
    Self::Dizzy,
    Self::Psychotic,
  ];

  /// The SDK's token for the room, `eax_environment` (`ShaderEditor/SHSoundEnvTools.cpp`), or the number for one EAX
  /// has no name for.
  pub fn label(self) -> String {
    String::from(match self {
      Self::Generic => "Generic",
      Self::PaddedCell => "Padded Cell",
      Self::Room => "Room",
      Self::Bathroom => "Bathroom",
      Self::LivingRoom => "Livingroom",
      Self::StoneRoom => "Stone Room",
      Self::Auditorium => "Auditorium",
      Self::ConcertHall => "Concert Hall",
      Self::Cave => "Cave",
      Self::Arena => "Arena",
      Self::Hangar => "Hangar",
      Self::CarpetedHallway => "Carpeted Hallway",
      Self::Hallway => "Hallway",
      Self::StoneCorridor => "Stone Corridor",
      Self::Alley => "Alley",
      Self::Forest => "Forest",
      Self::City => "City",
      Self::Mountains => "Mountains",
      Self::Quarry => "Quarry",
      Self::Plain => "Plain",
      Self::ParkingLot => "Parkinglot",
      Self::SewerPipe => "Sewer Pipe",
      Self::Underwater => "Under Water",
      Self::Drugged => "Drugged",
      Self::Dizzy => "Dizzy",
      Self::Psychotic => "Psychotic",
      Self::Unknown(number) => return number.to_string(),
    })
  }
}

impl Default for EaxEnvironment {
  /// `EAXLISTENER_DEFAULTENVIRONMENT`, which `CSoundRender_Environment::set_default` sets.
  fn default() -> Self {
    Self::Generic
  }
}

impl From<u32> for EaxEnvironment {
  fn from(number: u32) -> Self {
    usize::try_from(number)
      .ok()
      .and_then(|index| Self::NAMED.get(index).copied())
      .unwrap_or(Self::Unknown(number))
  }
}

impl From<EaxEnvironment> for u32 {
  fn from(environment: EaxEnvironment) -> Self {
    match environment {
      EaxEnvironment::Unknown(number) => number,
      named => EaxEnvironment::NAMED
        .iter()
        .position(|candidate| *candidate == named)
        .and_then(|index| u32::try_from(index).ok())
        .expect("every named room is in the numbering"),
    }
  }
}
