use serde::{Deserialize, Serialize};

/// What a sound means to the AI hearing it, `ESoundTypes` (`xrServerEntities/ai_sounds.h`): a category bit
/// (`SOUND_TYPE_ITEM`, `SOUND_TYPE_WEAPON`, ...) with an action bit. The named ones are those the SDK's sound editor
/// offers, `anomaly_type_token` (`xrGame/ai_sounds.cpp`), in its order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum XRaySoundGameType {
  /// `sg_Undefined`: the AI hears nothing it tells apart.
  Undefined,
  ItemPickingUp,
  ItemDropping,
  ItemTaking,
  ItemHiding,
  ItemUsing,
  WeaponShooting,
  WeaponEmptyClicking,
  WeaponBulletHit,
  WeaponRecharging,
  NpcDying,
  NpcInjuring,
  NpcStep,
  NpcTalking,
  NpcAttacking,
  NpcEating,
  AnomalyIdle,
  ObjectBreaking,
  ObjectColliding,
  ObjectExploding,
  /// The one type the sound renderer itself tells apart: it never occludes an ambient sound
  /// (`CSoundRender_Emitter::update`).
  WorldAmbient,
  /// A mask the SDK has no name for, kept as it was read so a sound written back says what it said.
  Unknown(u32),
}

impl XRaySoundGameType {
  /// Every type the SDK names, in the order its sound editor offers them.
  pub const NAMED: [Self; 21] = [
    Self::Undefined,
    Self::ItemPickingUp,
    Self::ItemDropping,
    Self::ItemTaking,
    Self::ItemHiding,
    Self::ItemUsing,
    Self::WeaponShooting,
    Self::WeaponEmptyClicking,
    Self::WeaponBulletHit,
    Self::WeaponRecharging,
    Self::NpcDying,
    Self::NpcInjuring,
    Self::NpcStep,
    Self::NpcTalking,
    Self::NpcAttacking,
    Self::NpcEating,
    Self::AnomalyIdle,
    Self::ObjectBreaking,
    Self::ObjectColliding,
    Self::ObjectExploding,
    Self::WorldAmbient,
  ];

  /// The `ESoundTypes` mask the sound stores, its category bit with its action bit.
  pub const fn get_mask(self) -> u32 {
    match self {
      Self::Undefined => 0x0000_0000,
      Self::ItemPickingUp => 0x4400_0000,
      Self::ItemDropping => 0x4200_0000,
      Self::ItemTaking => 0x4080_0000,
      Self::ItemHiding => 0x4100_0000,
      Self::ItemUsing => 0x4040_0000,
      Self::WeaponShooting => 0x8020_0000,
      Self::WeaponEmptyClicking => 0x8010_0000,
      Self::WeaponBulletHit => 0x8008_0000,
      Self::WeaponRecharging => 0x8004_0000,
      Self::NpcDying => 0x2002_0000,
      Self::NpcInjuring => 0x2001_0000,
      Self::NpcStep => 0x2000_8000,
      Self::NpcTalking => 0x2000_4000,
      Self::NpcAttacking => 0x2000_2000,
      Self::NpcEating => 0x2000_1000,
      Self::AnomalyIdle => 0x1000_0800,
      Self::ObjectBreaking => 0x0800_0400,
      Self::ObjectColliding => 0x0800_0200,
      Self::ObjectExploding => 0x0800_0100,
      Self::WorldAmbient => 0x0800_0080,
      Self::Unknown(mask) => mask,
    }
  }

  /// The SDK's token for the type, or the mask in hexadecimal for one it has no name for.
  pub fn label(self) -> String {
    String::from(match self {
      Self::Undefined => "undefined",
      Self::ItemPickingUp => "Item picking up",
      Self::ItemDropping => "Item dropping",
      Self::ItemTaking => "Item taking",
      Self::ItemHiding => "Item hiding",
      Self::ItemUsing => "Item using",
      Self::WeaponShooting => "Weapon shooting",
      Self::WeaponEmptyClicking => "Weapon empty clicking",
      Self::WeaponBulletHit => "Weapon bullet hit",
      Self::WeaponRecharging => "Weapon recharging",
      Self::NpcDying => "NPC dying",
      Self::NpcInjuring => "NPC injuring",
      Self::NpcStep => "NPC step",
      Self::NpcTalking => "NPC talking",
      Self::NpcAttacking => "NPC attacking",
      Self::NpcEating => "NPC eating",
      Self::AnomalyIdle => "Anomaly idle",
      Self::ObjectBreaking => "Object breaking",
      Self::ObjectColliding => "Object colliding",
      Self::ObjectExploding => "Object exploding",
      Self::WorldAmbient => "World ambient",
      Self::Unknown(mask) => return format!("{mask:#010x}"),
    })
  }
}

impl Default for XRaySoundGameType {
  /// `SoundSourceInfo::gameType` starts zeroed, and so does `ESoundThumbnail`'s.
  fn default() -> Self {
    Self::Undefined
  }
}

impl From<u32> for XRaySoundGameType {
  fn from(mask: u32) -> Self {
    Self::NAMED
      .into_iter()
      .find(|game_type| game_type.get_mask() == mask)
      .unwrap_or(Self::Unknown(mask))
  }
}

impl From<XRaySoundGameType> for u32 {
  fn from(game_type: XRaySoundGameType) -> Self {
    game_type.get_mask()
  }
}
