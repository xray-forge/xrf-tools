use uuid::{Uuid, uuid};

use crate::chunks::spawn_alife_spawns_chunk::SpawnALifeSpawnsChunk;
use crate::chunks::spawn_artefact_spawns_chunk::SpawnArtefactSpawnsChunk;
use crate::chunks::spawn_graphs_chunk::SpawnGraphsChunk;
use crate::chunks::spawn_header_chunk::SpawnHeaderChunk;
use crate::chunks::spawn_patrols_chunk::SpawnPatrolsChunk;
use crate::data::graph::graph_header::GraphHeader;
use crate::data::meta::alife_class::AlifeClass;
use crate::spawn_file::SpawnFile;

/// A root spawn holding nothing: no objects, levels, artefact points, patrols or graph vertices.
pub(crate) struct EmptySpawnFile;

impl EmptySpawnFile {
  pub(crate) fn build() -> SpawnFile {
    let guid: Uuid = uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7");

    SpawnFile {
      header: SpawnHeaderChunk {
        version: 10,
        guid,
        graph_guid: guid,
        objects_count: 0,
        levels_count: 0,
      },
      alife_spawn: SpawnALifeSpawnsChunk { objects: Vec::new() },
      artefact_spawn: SpawnArtefactSpawnsChunk { nodes: Vec::new() },
      patrols: SpawnPatrolsChunk { patrols: Vec::new() },
      graphs: SpawnGraphsChunk {
        header: GraphHeader {
          version: 10,
          vertices_count: 0,
          edges_count: 0,
          points_count: 0,
          guid,
          levels_count: 0,
        },
        levels: Vec::new(),
        vertices: Vec::new(),
        edges: Vec::new(),
        points: Vec::new(),
        cross_tables: Vec::new(),
      },
    }
  }
}

/// Packets the game wrote for new objects, created through `alife():create` in a running XRF game and written by
/// `STATE_Write` and `UPDATE_Write` before any went online, so xrf-engine's script classes wrote their tails and no
/// skeleton carried saved bones. No shipped spawn set holds these classes, which makes these the bytes to read against.
pub(crate) struct GameWrittenPackets;

impl GameWrittenPackets {
  /// A new object of each class Call of Pripyat creates from a spawn and no shipped spawn holds, but the stalker and
  /// the monster: the section it was created from, its class, and its spawn and update packets.
  pub(crate) const CLASSES: &'static [(&'static str, AlifeClass, &'static str, &'static str)] = &[
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
      "m_phantom",
      AlifeClass::CseAlifeCreaturePhantom,
      concat!(
        "d40000000000010000001fac0400baffffff3b2064656661756c745f637573746f6d5f646174612e6c747820402067656e65726174",
        "656420546875204a756c20323020323032332032313a30373a333020474d542b3033303020284561737465726e204575726f706561",
        "6e2053756d6d65722054696d65290d0a00ffffffffffffffff00002700000000803f0000000000000000ffff0000000000000000",
      ),
      "0000803f118ec238000000c8c30000a04100002f4400000000000000000000000000000000270000",
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

  pub(crate) const STALKER_SPAWN: &'static str = concat!(
    "4801000073696d5f64656661756c745f7374616c6b65725f305f64656661756c745f31000100000073696d5f64656661756c745f7374",
    "616c6b65725f3000090000001900000000000000cfe5f2fceae020cfe0f6feea000100d40000000000010000006dbb0400bfffffff3b",
    "2064656661756c745f637573746f6d5f646174612e6c747820402067656e65726174656420546875204a756c20323020323032332032",
    "313a30373a333020474d542b3033303020284561737465726e204575726f7065616e2053756d6d65722054696d65290d0a0d0a5b6761",
    "6d655f696e666f5d0d0a00ffffffffffffffff6163746f72735c7374616c6b65725f6e65757472616c5c7374616c6b65725f6e657574",
    "72616c5f3100000900000000803f0000000000000000ffff00000000000000000000ffff00050000000201020102040000000201020224",
    "656469746f720000ffff6e696c006e696c0000",
  );

  pub(crate) const STALKER_UPDATE: &'static str =
    "0000803fea792c38000000c8c30000a04100002f4400000000000000000000000000000000090000ffffffff000000000000000000";

  pub(crate) const MONSTER_SPAWN: &'static str = concat!(
    "d40000000000010000006dbb0400bfffffff3b2064656661756c745f637573746f6d5f646174612e6c747820402067656e65726174",
    "656420546875204a756c20323020323032332032313a30373a333020474d542b3033303020284561737465726e204575726f7065616e",
    "2053756d6d65722054696d65290d0a00ffffffffffffffff6d6f6e73746572735c646f675c646f675f776869746500002b0000000080",
    "3f0000000000000000ffff00000000000000000000ffff0024656469746f720000ffffffff6e696c006e696c00",
  );

  pub(crate) const MONSTER_UPDATE: &'static str =
    "0000803ffa405338000000c8c30000a04100002f44000000000000000000000000000000002b0000ffffffff0000000000000000";

  /// The bytes a packet's hex spells.
  pub(crate) fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
      .step_by(2)
      .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).expect("a fixture packet is hex"))
      .collect()
  }
}

/// Spawn packet states of Clear Sky's level lists (`gamedata-cs/levels/<level>/level.spawn`), script version 8, each
/// with the `u16` size `Spawn_Write` patches in before its fields: `agroprom`'s at spawn version 124, the others at the
/// version their name ends with.
pub(crate) struct ClearSkyStates;

impl ClearSkyStates {
  /// `agr_smart_cover`: 77 bytes ending at the combat cover flag, with no `can_fire` and no script fields.
  pub(crate) const SMART_COVER: &'static str = concat!(
    "4d00ffff0000000001000000ffffffffbfffffff00ffffffffffffffff",
    "010000000000000000000000000000000040",
    "636f7665725f6c6f6f70686f6c655f3200",
    "00000000000070410000204101",
  );

  /// `agr_smart_terrain_1_2_box`: 77 bytes, the size, 27 of the abstract object and 48 of its visual, with no access
  /// fields or tip.
  pub(crate) const INVENTORY_BOX: &'static str = concat!(
    "4d00ffff0000000001000000ffffffff3fffffff00ffffffffffffffff",
    "64796e616d6963735c65717569706d656e745f63616368655c65717569706d656e745f626f785f30325f636173650000",
  );

  /// `clmbl#0`: 79 bytes, the size, 27 of the abstract object, then one shape, a box of 48 bytes, with no material.
  pub(crate) const CLIMABLE: &'static str = concat!(
    "4f00ffff0000000001000000ffffffffbfffffff00ffffffffffffffff",
    "0101",
    "6901b6400000000000000000000000000000363b0000000000000000000000008089eb3e000000000000000000000000",
  );

  /// `escape`'s `esc_medkit`: 81 bytes, the size, 27 of the abstract object, 48 of its visual and the condition, with
  /// no upgrades.
  pub(crate) const ITEM_123: &'static str = concat!(
    "5100ffff0000000001000000ffffffff3fffffff00ffffffffffffffff",
    "64796e616d6963735c646576696365735c6465765f6170746563686b615c6465765f6170746563686b615f6c6f770000",
    "0000803f",
  );

  /// `escape`'s `esc_wpn_ak74u`: 79 bytes, the size, 27 of the abstract object, 38 of its visual and the condition,
  /// with no upgrades, then the weapon's seven bytes and its loaded grenades.
  pub(crate) const WEAPON_123: &'static str = concat!(
    "4f00ffff0000000001000000ffffffff3fffffff00ffffffffffffffff",
    "64796e616d6963735c776561706f6e735c77706e5f616b3734755c77706e5f616b3734750000",
    "0000803f",
    "5a00000000000000",
  );

  /// `mp_garbage`'s `mp_garbage_mp_wpn_rg-6`: 74 bytes, the size, 27 of the abstract object, 34 of its visual and the
  /// condition, with no upgrades, then the weapon's seven bytes and no loaded grenades.
  pub(crate) const WEAPON_122: &'static str = concat!(
    "4a00ffff0000000001000000ffffffffbfffffff00ffffffffffffffff",
    "64796e616d6963735c776561706f6e735c77706e5f7267365c77706e5f7267360000",
    "0000803f",
    "5a000100000000",
  );

  /// `escape`'s `esc_dog_weak`, spawn version 123: 96 bytes, the monster's engine fields, then `se_monster`'s job kind
  /// for `nil`, its squad as `nil` and its forced online flag.
  pub(crate) const MONSTER: &'static str = concat!(
    "6000ffff0000000001000000ffffffffbfffffff00ffffffffffffffff",
    "6d6f6e73746572735c646f675c646f6700000000000000000000000000000000",
    "00ffff00000000000000000000ffff0024656469746f720000ffffffff",
    "026e696c0000",
  );

  /// `escape`'s `esc_flashman_1`, spawn version 123: 171 bytes, the stalker's engine fields with no corpse access
  /// fields, then `se_stalker`'s job kind for `nil`, its loot flag, its squad as `nil` and its forced online flag.
  pub(crate) const STALKER: &'static str = concat!(
    "ab008813000000010000006573635f666c6173686d616e00ffffffff010000800100008000ffff0000000001000000ffffffff3fffffff",
    "00b4010000ffffffff6163746f72735c7374616c6b65725f736f6c646965725c7374616c6b65725f736f6c646965725f310000000000",
    "000000000000000000000000ffff00000000000000000000ffff00050000000102020101040000000200010224656469746f720000ffff",
    "02006e696c0000",
  );

  /// `darkvalley`'s `val_sim_faction_killer`, spawn version 123: 119 bytes, the smart zone, then `se_sim_faction`'s
  /// three bytes, its `nil` last spawn time, four empty tables and its marker.
  pub(crate) const SIM_FACTION: &'static str = concat!(
    "7700ffff0000000001000000ffffffff3effffff5b66616374696f6e5d0d0a6e616d65203d206b696c6c65720d0a73657474696e67",
    "73203d206d6973635c66616374696f6e5f6b696c6c65722e6c747800ffffffffffffffff01000000000000000000000000000000803f",
    "00000000ff00000000000900",
  );

  /// `marsh`'s `mar_smart_terrain_csky_base`, spawn version 123: 200 bytes, the smart zone, then `se_smart_terrain`'s
  /// combat manager with its cover manager, two empty tables and the markers closing each.
  pub(crate) const SMART_TERRAIN: &'static str = concat!(
    "c800ffff0000000001000000ffffffff3effffff5b736d6172745f7465727261696e5d0d0a74617267657473203d200d0a73696d5f74",
    "797065203d20626173650d0a73717561645f6361706163697479203d20330d0a73717561645f6964203d2033310d0a6163746f725f6869",
    "646573203d206d61725f696e74726f5f7a6f6e650d0a6e6f5f6d7574616e74203d207472756500ffffffffffffffff01000000000000",
    "00000000000000a379474000006e696c00ffff6e696c000000000000000200130000001700",
  );

  /// `marsh`'s `level_prefix_actor`, spawn version 123: 302 bytes, the actor's engine fields, then `se_actor`'s random
  /// task manager, squad id generator and minigames manager as a new game holds them, each closed by its marker.
  pub(crate) const ACTOR: &'static str = concat!(
    "2e01ffff0000000001000000ffffffffbfffffff00ffffffffffffffff6163746f72735c7374616c6b65725f6865726f5c7374616c6b",
    "65725f6865726f5f3100000000000000803f0000000000000000ffff00000000000000002800000000000000006163746f7200ffffff",
    "ff01000080010000800024656469746f720000ffffffff0000000000000000000008000000000f0000000000000000000800020074",
    "6573745f63726f776b696c6c657200434d4743726f774b696c6c65720076616c6961626c6500003c000004000000000a0064000000",
    "0000000a0000001600746573745f73686f6f74696e6700434d4753686f6f74696e670076616c6961626c650000006e696c006e696c",
    "006e696c006e696c006e696c000100000000000000000000006e696c0026008c00a900",
  );
}
