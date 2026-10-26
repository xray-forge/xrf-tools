use crate::data::meta::alife_class::AlifeClass;

/// Packets the game wrote for new objects, created through `alife():create` in a running XRF game and written by
/// `STATE_Write` and `UPDATE_Write` before any went online, so xrf-engine's script classes wrote their tails and no
/// skeleton carried saved bones. No shipped spawn set holds these classes, which makes these the bytes to read against.
pub(crate) struct GameWrittenPackets;

impl GameWrittenPackets {
  /// A new object of each class Call of Pripyat creates from a spawn and no shipped spawn holds, but the stalker and
  /// the monster: the section it was created from, its class, and its spawn and update packets.
  pub const CLASSES: &'static [(&'static str, AlifeClass, &'static str, &'static str)] = &[
    (
      "bolt",
      AlifeClass::CseAlifeItemBolt,
      concat!(
        "d40000000000010000001fac0400baffffff00ffffffffffffffff64796e616d6963735c646576696365735c6465765f626f6c745c",
        "6465765f626f6c7400000000803f00000000",
      ),
      "00",
    ),
    (
      "device_torch",
      AlifeClass::CseAlifeItemTorch,
      concat!(
        "d40000000000010000001fac0400bfffffff00ffffffffffffffff64796e616d6963735c646576696365735c6465765f746f726368",
        "5f6c696768745c6465765f746f7263685f6c6967687400000000803f00000000",
      ),
      "0000",
    ),
    (
      "document",
      AlifeClass::CseAlifeItemDocument,
      concat!(
        "d40000000000010000001fac0400bfffffff00ffffffffffffffff64796e616d6963735c65717569706d656e74735c6974656d5f64",
        "6f63756d656e745f3100000000803f0000000000",
      ),
      "00",
    ),
    (
      "device_flare",
      AlifeClass::CseAlifeItem,
      concat!(
        "d40000000000010000001fac0400bfffffff00ffffffffffffffff64796e616d6963735c646576696365735c6465765f666c617265",
        "5c6465765f666c61726500000000803f00000000",
      ),
      "00",
    ),
    (
      "m_crow",
      AlifeClass::CseAlifeCreatureCrow,
      concat!(
        "d40000000000010000001fac0400baffffff00ffffffffffffffff6d6f6e73746572735c63726f775c63726f7700003600000000803f",
        "0000000000000000ffff0000000000000000",
      ),
      "0000803f3c402a3a000000c8c30000a04100002f4400000000000000000000000000000000360000",
    ),
    (
      "script_object",
      AlifeClass::CseAlifeDynamicObjectVisual,
      "d40000000000010000001fac0400bfffffff00ffffffffffffffff0000",
      "",
    ),
    (
      "search_light",
      AlifeClass::CseAlifeObjectProjector,
      concat!(
        "d40000000000010000001fac0400baffffff00ffffffffffffffff64796e616d6963735c6c696768745c70726f6a6563746f7200",
        "00",
      ),
      "",
    ),
    (
      "ph_skeleton_object",
      AlifeClass::CseAlifePhSkeletonObject,
      "d40000000000010000001fac0400baffffff00ffffffffffffffff000024656469746f720000ffff",
      "",
    ),
    (
      "stationary_mgun",
      AlifeClass::CseAlifeStationaryMGun,
      concat!(
        "d40000000000010000001fac0400bfffffff00ffffffffffffffff64796e616d6963735c776561706f6e735c77706e5f6d6f756e74",
        "65645c77706e5f6d6f756e7465645f706b6d5f7472656e6f67610000",
      ),
      // The gun's working byte and aim, which the engine never initialised.
      "01db0400003183793900000000",
    ),
    (
      "simulation_stalker",
      AlifeClass::SimSquadScripted,
      "ffff00000000010000001fac04003fffffff00ffffffffffffffff000000006e696c006e696c006e696c006e696c001000",
      "",
    ),
  ];

  pub const STALKER_SPAWN: &'static str = concat!(
    "4801000073696d5f64656661756c745f7374616c6b65725f305f64656661756c745f31000100000073696d5f64656661756c745f7374",
    "616c6b65725f3000090000001900000000000000cfe5f2fceae020cfe0f6feea000100d40000000000010000006dbb0400bfffffff3b",
    "2064656661756c745f637573746f6d5f646174612e6c747820402067656e65726174656420546875204a756c20323020323032332032",
    "313a30373a333020474d542b3033303020284561737465726e204575726f7065616e2053756d6d65722054696d65290d0a0d0a5b6761",
    "6d655f696e666f5d0d0a00ffffffffffffffff6163746f72735c7374616c6b65725f6e65757472616c5c7374616c6b65725f6e657574",
    "72616c5f3100000900000000803f0000000000000000ffff00000000000000000000ffff00050000000201020102040000000201020224",
    "656469746f720000ffff6e696c006e696c0000",
  );

  pub const STALKER_UPDATE: &'static str =
    "0000803fea792c38000000c8c30000a04100002f4400000000000000000000000000000000090000ffffffff000000000000000000";

  pub const MONSTER_SPAWN: &'static str = concat!(
    "d40000000000010000006dbb0400bfffffff3b2064656661756c745f637573746f6d5f646174612e6c747820402067656e65726174",
    "656420546875204a756c20323020323032332032313a30373a333020474d542b3033303020284561737465726e204575726f7065616e",
    "2053756d6d65722054696d65290d0a00ffffffffffffffff6d6f6e73746572735c646f675c646f675f776869746500002b0000000080",
    "3f0000000000000000ffff00000000000000000000ffff0024656469746f720000ffffffff6e696c006e696c00",
  );

  pub const MONSTER_UPDATE: &'static str =
    "0000803ffa405338000000c8c30000a04100002f44000000000000000000000000000000002b0000ffffffff0000000000000000";

  /// The bytes a packet's hex spells.
  pub fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
      .step_by(2)
      .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).expect("a fixture packet is hex"))
      .collect()
  }
}
