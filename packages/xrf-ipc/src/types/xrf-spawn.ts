// Auto-generated rust bindings. Do not edit it manually.

import { Affine3d, Sphere, Vector3d } from "#/types/xrf-math";

/** Every `type` the `ActorPhysicsState` union is told apart by, so a switch or a comparison names one. */
export enum EActorPhysicsState {
  /** No body was saved. */
  ABSENT = "Absent",
  /** The living body, `m_AliveState`, in the order the actor writes it. */
  ALIVE = "Alive",
  /**
   * A dead body, `m_DeadBodyData` as `CActor::net_ExportDeadBody` writes it: the low and high corners its bones are
   * quantised within, then `bone_data_size` bytes for each of its `bone_count` bones.
   */
  DEAD = "Dead",
}

/**
 * The body an actor's update packet saves, behind the `u16` count `CSE_ALifeCreatureActor::UPDATE_Write` opens it with:
 * nothing at zero, the living body's physics state at one, and a dead body's bones above that.
 */
export type ActorPhysicsState =
  /** No body was saved. */
  | { type: "Absent" }
  /** The living body, `m_AliveState`, in the order the actor writes it. */
  | {
      type: "Alive";
      /** The body's enabled flag, as the byte the engine reads it into. */
      enabled: number;
      angularVelocity: [RawFloat, RawFloat, RawFloat];
      linearVelocity: [RawFloat, RawFloat, RawFloat];
      force: [RawFloat, RawFloat, RawFloat];
      torque: [RawFloat, RawFloat, RawFloat];
      position: [RawFloat, RawFloat, RawFloat];
      /** The orientation quaternion as x, y, z and w. */
      rotation: [RawFloat, RawFloat, RawFloat, RawFloat];
    }
  /**
   * A dead body, `m_DeadBodyData` as `CActor::net_ExportDeadBody` writes it: the low and high corners its bones are
   * quantised within, then `bone_data_size` bytes for each of its `bone_count` bones.
   */
  | {
      type: "Dead";
      boneCount: number;
      boneDataSize: number;
      lowCorner: [RawFloat, RawFloat, RawFloat];
      highCorner: [RawFloat, RawFloat, RawFloat];
      bones: Array<number>;
    };

export type AlifeActor = {
  base: AlifeObjectActor;
  /** The script class's own field, `se_actor` in vanilla. */
  startPositionFilled: number;
};

export type AlifeAnomalousZone = {
  base: AlifeObjectAnomalyZone;
  lastSpawnTime: LastSpawnTime;
};

/**
 * The actor as Clear Sky's `se_actor` saves it (`gamedata-cs/scripts/se_actor.script`): the server class's fields, then,
 * closed by a save marker, the random task manager, the squads' id generator and the minigames manager, each closed by
 * its own marker. It adds nothing to the update packet.
 *
 * The task manager (`CRandomTask:save`, `task_manager.script`) keeps the tasks given, the rewards and the upgrade
 * tasks, which a running game fills; a spawn holds none, so their counts are read as empty and any other is refused
 * rather than read, and only its task id generator is kept.
 */
export type AlifeClearSkyActor = {
  base: AlifeObjectActor;
  /** The task manager's id generator. */
  taskIds: ClearSkyObjectCollection;
  /** The simulation board's id generator, which numbers squads. */
  squadIds: ClearSkyObjectCollection;
  minigames: Array<ClearSkyMinigame>;
};

/**
 * A monster as Clear Sky's `se_monster` saves one (`gamedata-cs/scripts/se_monster.script`): the server class's spawn
 * fields, then whether its job keeps it online, the squad it belongs to and whether the simulation forces it online,
 * unmarked, adding nothing to the update packet.
 */
export type AlifeClearSkyMonster = {
  base: AlifeObjectMonsterBase;
  jobOnline: ClearSkyJobOnline;
  /** `squad_id` as the script writes it, `tostring` of the squad's id; `None` for `nil`. */
  squadId: string | null;
  /** `sim_forced_online`, a `w_bool`. */
  simForcedOnline: number;
};

/**
 * A faction as Clear Sky's `se_sim_faction` saves one (`gamedata-cs/scripts/sim_faction.script`), the class Clear Sky
 * registers for `SFACTION`: the smart zone's spawn fields, then, closed by a save marker, whether the player leads it,
 * whether its start position was filled and its expansion level, followed by its last spawn time and four tables.
 *
 * The last spawn time, the squads' targets, the random tasks, the attack counts and the squads are what a running
 * simulation fills; a spawn holds `nil` and four empty tables, so those are read as such and any other value is refused
 * rather than read. The time is `utils.w_CTime`'s Clear Sky form, whose `nil` is a lone 255.
 */
export type AlifeClearSkySimFaction = {
  base: AlifeSmartZone;
  /** `community_player`, a `w_bool`. */
  communityPlayer: number;
  /** `start_position_filled`, a `w_bool`. */
  startPositionFilled: number;
  currentExpansionLevel: number;
};

/**
 * A smart terrain as Clear Sky's `se_smart_terrain` saves one (`gamedata-cs/scripts/smart_terrain.script`): the smart
 * zone's spawn fields, then, closed by a save marker, its combat manager and the stalkers' jobs and the jobs' death
 * times. The jobs and death times are records only a running simulation fills; a spawn holds none, so their counts are
 * read as empty and any other value is refused rather than read.
 */
export type AlifeClearSkySmartTerrain = {
  base: AlifeSmartZone;
  combatManager: ClearSkyCombatManager;
};

/**
 * A stalker as Clear Sky's `se_stalker` saves one (`gamedata-cs/scripts/se_stalker.script`): the server class's spawn
 * fields, then whether its job keeps it online, whether its corpse's loot was dropped, the squad it belongs to and
 * whether the simulation forces it online, unmarked, adding nothing to the update packet.
 */
export type AlifeClearSkyStalker = {
  base: AlifeObjectHumanStalker;
  jobOnline: ClearSkyJobOnline;
  /** Whether the death manager already dropped the corpse's loot, `w_bool` of `death_droped`; zero for a new object. */
  lootDropped: number;
  /** `squad_id` as the script writes it, `tostring` of the squad's id; `None` for `nil`. */
  squadId: string | null;
  /** `sim_forced_online`, a `w_bool`. */
  simForcedOnline: number;
};

export type AlifeGraphPoint = {
  connectionPointName: string;
  connectionLevelName: string;
  location0: number;
  location1: number;
  location2: number;
  location3: number;
};

export type AlifeLevelChanger = {
  base: AlifeObjectSpaceRestrictor;
  destGameVertexId: number;
  destLevelVertexId: number;
  destPosition: Vector3d;
  destDirection: Vector3d;
  destLevelName: string;
  destGraphPoint: string;
  silentMode: number;
  /** The script class's own fields, `se_level_changer` in vanilla and `LevelChanger` in xrf-engine. */
  enabled: number;
  hint: string;
};

/**
 * A monster as its script class `se_monster` saves one: vanilla's `se_monster.script` and xrf-engine's `Monster.ts`
 * both follow the server class's spawn fields with the offline state alone, unmarked, and add nothing to the update
 * packet.
 *
 * The tail mirrors `STATE_Write` in xrf-engine's `src/engine/core/objects/creature/Monster.ts`, whose own test pins
 * its write order, so a change to either side changes the other in the same work.
 */
export type AlifeMonster = {
  base: AlifeObjectMonsterBase;
  offlineState: OfflineObjectState;
};

/**
 * An object of a spawn set (`all.spawn`): its spawn packet, the spawn packet header's fields then the class's spawn
 * fields, and its update packet, the class's update fields, each in a chunk of its own behind a `u16` length.
 */
export type AlifeObject = {
  clsid: ClsId;
  /**
   * The server class payload, its fields from both packets. The engine applies the update packet over the spawn
   * packet when it creates the object, so where both carry a field the update packet's is the one the game uses.
   */
  inherited: AlifeObjectInherited;
} & SpawnPacketHeader;

/** Generic ALife object abstraction data. */
export type AlifeObjectAbstract = {
  gameVertexId: number;
  distance: number | null;
  directControl: number;
  levelVertexId: number;
  flags: number;
  customData: string;
  storyId: number;
  spawnStoryId: number;
};

/**
 * The actor, `CSE_ALifeCreatureActor`: a creature and a trader with a skeleton. Its update packet follows the
 * creature's fields with its own movement and the body it saved; the trader and the skeleton add none.
 */
export type AlifeObjectActor = {
  base: AlifeObjectCreature;
  trader: AlifeObjectTraderAbstract;
  skeleton: AlifeObjectSkeleton;
  holderId: number;
  /** The movement state flags, `mstate`. */
  updateMovementState: number;
  updateAcceleration: ScaledDirection;
  updateVelocity: ScaledDirection;
  updateRadiation: number | null;
  /** The weapon slot in hand, `weapon`. */
  updateWeapon: number;
  updateBody: ActorPhysicsState;
};

export type AlifeObjectAnomalyZone = {
  base: AlifeObjectCustomZone;
  offlineInteractiveRadius: number | null;
  artefactSpawnCount: number;
  artefactPositionOffset: number;
};

export type AlifeObjectBreakable = {
  base: AlifeObjectDynamicVisual;
  health: number | null;
};

export type AlifeObjectCar = {
  base: AlifeObjectDynamicVisual;
  skeleton: AlifeObjectSkeleton;
  health: number | null;
};

/**
 * `CSE_ALifeObjectClimable`. Up to spawn version 126 the packet stores no game material, which then holds the
 * constructor's `materials\fake_ladders` (`xrServer_Objects_ALife.cpp:1727`).
 */
export type AlifeObjectClimable = {
  base: AlifeObjectShape;
  gameMaterial: string;
};

/**
 * A creature, `CSE_ALifeCreatureAbstract`. Its update packet repeats its health, team, squad and group, and carries
 * where it stands and which way its torso faces; the engine reads them over the spawn packet's, so a creature starts
 * where its update packet says.
 */
export type AlifeObjectCreature = {
  base: AlifeObjectDynamicVisual;
  team: number;
  squad: number;
  group: number;
  health: number | null;
  dynamicOutRestrictions: Array<number>;
  dynamicInRestrictions: Array<number>;
  killerId: number;
  gameDeathTime: number;
  updateHealth: number | null;
  updateTimestamp: number;
  updateFlags: number;
  /** Where the creature stands, `o_Position`. */
  updatePosition: Vector3d;
  /** The model's heading, `o_model`. */
  updateModelYaw: number | null;
  /** The torso's heading, `o_torso.yaw`, which the spawn packet's direction gives as its y. */
  updateTorsoYaw: number | null;
  /** The torso's pitch, `o_torso.pitch`, which the spawn packet's direction gives as its x. */
  updateTorsoPitch: number | null;
  updateTorsoRoll: number | null;
  updateTeam: number;
  updateSquad: number;
  updateGroup: number;
};

export type AlifeObjectCustomZone = {
  base: AlifeObjectSpaceRestrictor;
  maxPower: number | null;
  ownerId: number;
  enabledTime: number;
  disabledTime: number;
  startTimeShift: number;
};

export type AlifeObjectDynamic = {
  base: AlifeObjectAbstract;
};

export type AlifeObjectDynamicVisual = {
  base: AlifeObjectAbstract;
  visualName: string;
  visualFlags: number;
};

export type AlifeObjectHangingLamp = {
  base: AlifeObjectDynamicVisual;
  skeleton: AlifeObjectSkeleton;
  mainColor: number;
  mainBrightness: number | null;
  colorAnimator: string;
  mainRange: number | null;
  lightFlags: number;
  startupAnimation: string;
  fixedBones: string;
  health: number | null;
  virtualSize: number | null;
  ambientRadius: number | null;
  ambientPower: number | null;
  ambientTexture: string;
  lightTexture: string;
  lightBone: string;
  spotConeAngle: number | null;
  glowTexture: string;
  glowRadius: number | null;
  lightAmbientBone: string;
  volumetricQuality: number | null;
  volumetricIntensity: number | null;
  volumetricDistance: number | null;
};

export type AlifeObjectHelicopter = {
  base: AlifeObjectDynamicVisual;
  motion: AlifeObjectMotion;
  skeleton: AlifeObjectSkeleton;
  startupAnimation: string;
  engineSound: string;
};

/**
 * A human, `CSE_ALifeHumanAbstract`: a trader and a monster, in that order, so unlike the actor's its spawn packet
 * opens with the trader's fields, ahead of every field the object inherits. The human brain follows them with the
 * equipment and weapon preferences it picks at random. Its update packet is the monster's.
 */
export type AlifeObjectHumanAbstract = {
  trader: AlifeObjectTraderAbstract;
  base: AlifeObjectMonsterAbstract;
  /** `CALifeHumanBrain::m_cpEquipmentPreferences`, five values in 0..3 as the SDK writes them. */
  equipmentPreferences: Array<number>;
  /** `CALifeHumanBrain::m_cpMainWeaponPreferences`, four values in 0..3 as the SDK writes them. */
  mainWeaponPreferences: Array<number>;
};

/**
 * A stalker, `CSE_ALifeHumanStalker`: a human with a physics skeleton, whose update packet closes with the dialog it
 * starts a conversation with.
 */
export type AlifeObjectHumanStalker = {
  base: AlifeObjectHumanAbstract;
  skeleton: AlifeObjectSkeleton;
  /** `m_start_dialog`, which only the update packet stores; empty for the default. */
  updateStartDialog: string;
};

export type AlifeObjectInherited =
  | ({
      type: "SeActor";
    } & AlifeActor)
  | ({
      type: "SeStalker";
    } & AlifeStalker)
  | ({
      type: "SeMonster";
    } & AlifeMonster)
  | ({
      type: "CseAlifeObjectBreakable";
    } & AlifeObjectBreakable)
  | ({
      type: "CseAlifeObjectClimable";
    } & AlifeObjectClimable)
  | ({
      type: "CseAlifeGraphPoint";
    } & AlifeGraphPoint)
  | ({
      type: "CseAlifeSpaceRestrictor";
    } & AlifeObjectSpaceRestrictor)
  | ({
      type: "SeSmartCover";
    } & AlifeSmartCover)
  | ({
      type: "CseAlifeAnomalousZone";
    } & AlifeAnomalousZone)
  | ({
      type: "CseAlifeTorridZone";
    } & AlifeObjectTorridZone)
  | ({
      type: "SeSmartTerrain";
    } & AlifeSmartTerrain)
  | ({
      type: "SeLevelChanger";
    } & AlifeLevelChanger)
  | ({
      type: "CseAlifeTeamBaseZone";
    } & AlifeTeamBaseZone)
  | ({
      type: "CseAlifeZoneVisual";
    } & AlifeZoneVisual)
  | ({
      type: "CseAlifeCar";
    } & AlifeObjectCar)
  | ({
      type: "CseAlifeTrader";
    } & AlifeObjectTrader)
  | ({
      type: "CseAlifeObjectPhysic";
    } & AlifeObjectPhysic)
  | ({
      type: "CseAlifeHelicopter";
    } & AlifeObjectHelicopter)
  | ({
      type: "CseAlifeInventoryBox";
    } & AlifeObjectInventoryBox)
  | ({
      type: "CseAlifeObjectHangingLamp";
    } & AlifeObjectHangingLamp)
  | ({
      type: "CseAlifeItem";
    } & AlifeObjectItem)
  | ({
      type: "CseAlifeItemExplosive";
    } & AlifeObjectItemExplosive)
  | ({
      type: "CseAlifeItemPda";
    } & AlifeObjectItemPda)
  | ({
      type: "CseAlifeItemAmmo";
    } & AlifeObjectItemAmmo)
  | ({
      type: "CseAlifeItemGrenade";
    } & AlifeObjectItemGrenade)
  | ({
      type: "CseAlifeItemArtefact";
    } & AlifeObjectItemArtefact)
  | ({
      type: "CseAlifeItemWeapon";
    } & AlifeObjectItemWeapon)
  | ({
      type: "CseAlifeItemDetector";
    } & AlifeObjectItemDetector)
  | ({
      type: "CseAlifeItemHelmet";
    } & AlifeObjectItemHelmet)
  | ({
      type: "CseAlifeItemCustomOutfit";
    } & AlifeObjectItemCustomOutfit)
  | ({
      type: "CseAlifeItemWeaponShotgun";
    } & AlifeObjectItemWeaponShotgun)
  | ({
      type: "CseAlifeItemWeaponMagazined";
    } & AlifeObjectItemWeaponMagazined)
  | ({
      type: "CseAlifeItemWeaponMagazinedWGl";
    } & AlifeObjectItemWeaponMagazinedWgl)
  | ({
      type: "CseAlifeItemBolt";
    } & AlifeObjectItem)
  | ({
      type: "CseAlifeItemTorch";
    } & AlifeObjectItemTorch)
  | ({
      type: "CseAlifeItemDocument";
    } & AlifeObjectItemDocument)
  | ({
      type: "CseAlifeCreatureCrow";
    } & AlifeObjectCreature)
  | ({
      type: "CseAlifeCreaturePhantom";
    } & AlifeObjectCreature)
  | ({
      type: "CseAlifeDynamicObjectVisual";
    } & AlifeObjectDynamicVisual)
  | ({
      type: "CseAlifeObjectProjector";
    } & AlifeObjectDynamicVisual)
  | ({
      type: "CseAlifePhSkeletonObject";
    } & AlifeObjectPhSkeletonObject)
  | ({
      type: "CseAlifeStationaryMGun";
    } & AlifeObjectStationaryMgun)
  | ({
      type: "SimSquadScripted";
    } & AlifeSquad)
  | ({
      type: "ClearSkySeActor";
    } & AlifeClearSkyActor)
  | ({
      type: "ClearSkySeStalker";
    } & AlifeClearSkyStalker)
  | ({
      type: "ClearSkySeMonster";
    } & AlifeClearSkyMonster)
  | ({
      type: "ClearSkySeSmartTerrain";
    } & AlifeClearSkySmartTerrain)
  | ({
      type: "ClearSkySeSimFaction";
    } & AlifeClearSkySimFaction);

/**
 * `CSE_ALifeInventoryBox`. Up to spawn version 124 the packet stores none of its own fields, which then hold the
 * constructor's: open to taking, not closed, the `inventory_box_use` tip (`xrServer_Objects_ALife.cpp:1879`).
 */
export type AlifeObjectInventoryBox = {
  base: AlifeObjectDynamicVisual;
  canTake: number;
  isClosed: number;
  tip: string;
};

/**
 * An item, `CSE_ALifeItem`: a dynamic visual and an inventory item. Up to spawn version 123 the packet stores no
 * upgrades, and the item holds none, as `CSE_ALifeInventoryItem`'s constructor leaves `m_upgrades`
 * (`xrServer_Objects_ALife_Items.cpp:27`, read above 123 at `:92`).
 */
export type AlifeObjectItem = {
  base: AlifeObjectDynamicVisual;
  condition: number | null;
  /** The upgrade sections installed on the item (`CSE_ALifeInventoryItem::m_upgrades`). */
  upgrades: Array<string>;
  /**
   * The physics state the update packet carries, whose position the engine moves the item to over the spawn
   * packet's.
   */
  updateState: PhysicsNetState | null;
};

export type AlifeObjectItemAmmo = {
  base: AlifeObjectItem;
  ammoLeft: number;
  /** The rounds left as the update packet stores them, which the engine reads over `ammo_left`. */
  updateAmmoLeft: number;
};

/** An artefact, `CSE_ALifeItemArtefact`, which adds nothing to either packet of the item it is. */
export type AlifeObjectItemArtefact = {
  base: AlifeObjectItem;
};

/** An outfit, `CSE_ALifeItemCustomOutfit`, whose update packet repeats its condition after the item's. */
export type AlifeObjectItemCustomOutfit = {
  base: AlifeObjectItem;
  /** The condition as the update packet quantises it, which the engine reads over the spawn packet's. */
  updateCondition: number;
};

/** A detector, `CSE_ALifeItemDetector`, which adds nothing to either packet of the item it is. */
export type AlifeObjectItemDetector = {
  base: AlifeObjectItem;
};

/**
 * A document, `CSE_ALifeItemDocument`: an item whose spawn packet closes with the info portion reading it gives; its
 * update packet is the item's.
 */
export type AlifeObjectItemDocument = {
  base: AlifeObjectItem;
  /** The info portion, `m_wDoc`. */
  infoPortion: string;
};

/** An explosive, `CSE_ALifeItemExplosive`, which adds nothing to either packet of the item it is. */
export type AlifeObjectItemExplosive = {
  base: AlifeObjectItem;
};

/** A grenade, `CSE_ALifeItemGrenade`, which adds nothing to either packet of the item it is. */
export type AlifeObjectItemGrenade = {
  base: AlifeObjectItem;
};

/** A helmet, `CSE_ALifeItemHelmet`, whose update packet repeats its condition after the item's. */
export type AlifeObjectItemHelmet = {
  base: AlifeObjectItem;
  /** The condition as the update packet quantises it, which the engine reads over the spawn packet's. */
  updateCondition: number;
};

export type AlifeObjectItemPda = {
  base: AlifeObjectItem;
  owner: number;
  character: string;
  infoPortion: string;
};

/**
 * A torch, `CSE_ALifeItemTorch`, whose update packet closes with a byte of whether its light and night vision are on
 * and whether it is attached; its spawn packet is the item's.
 */
export type AlifeObjectItemTorch = {
  base: AlifeObjectItem;
  /** `eTorchActive`, `eNightVisionActive` and `eAttached` in its three low bits. */
  updateFlags: number;
};

/**
 * A weapon, `CSE_ALifeItemWeapon`. Its update packet repeats the rounds loaded, weapon state, addons and ammo type the
 * spawn packet stores, and the engine reads them over the spawn packet's, so those are what the game starts with. Up
 * to spawn version 122 the spawn packet stores no loaded grenades, which then hold the constructor's none of the first
 * type (`xrServer_Objects_ALife_Items.cpp:490-491`, read above 122 at `:565`), the byte `pack_to_byte` makes zero.
 */
export type AlifeObjectItemWeapon = {
  base: AlifeObjectItem;
  ammoCurrent: number;
  ammoElapsed: number;
  weaponState: number;
  addonFlags: number;
  ammoType: number;
  elapsedGrenades: number;
  /** The condition as the update packet quantises it, which the engine reads over the spawn packet's. */
  updateCondition: number;
  updateWeaponFlags: number;
  updateAmmoElapsed: number;
  updateAddonFlags: number;
  updateAmmoType: number;
  updateWeaponState: number;
  /** `m_bZoom`, nonzero while aiming. */
  updateZoom: number;
};

export type AlifeObjectItemWeaponMagazined = {
  base: AlifeObjectItemWeapon;
  /** The fire mode selected, `m_u8CurFireMode`. */
  updateFireMode: number;
};

export type AlifeObjectItemWeaponMagazinedWgl = {
  base: AlifeObjectItemWeaponMagazined;
  /** `m_bGrenadeMode`, nonzero while the launcher is selected. */
  updateGrenadeMode: number;
};

/** A shotgun, `CSE_ALifeItemWeaponShotGun`, which the engine derives from the magazined weapon. */
export type AlifeObjectItemWeaponShotgun = {
  base: AlifeObjectItemWeaponMagazined;
  /** The ammo type of each shell loaded, `m_AmmoIDs`, behind a byte counting them. */
  updateAmmoIds: Array<number>;
};

/**
 * A creature the alife simulation moves offline, `CSE_ALifeMonsterAbstract`: every monster and every stalker. Its
 * spawn packet adds the space restrictions it keeps to and the smart terrain it serves, and its update packet the game
 * graph edge it walks, after the creature's fields in each.
 */
export type AlifeObjectMonsterAbstract = {
  base: AlifeObjectCreature;
  /** The restrictors it may not leave, `m_out_space_restrictors`, as a comma-separated list of names. */
  outSpaceRestrictors: string;
  /** The restrictors it may not enter, `m_in_space_restrictors`. */
  inSpaceRestrictors: string;
  /** The smart terrain whose job it holds, `m_smart_terrain_id`; `u16::MAX` for none. */
  smartTerrainId: number;
  /** Whether it reached its smart terrain task, a C++ `bool` stored as one byte. */
  taskReached: number;
  /** The game vertex it walks to, `m_tNextGraphID`. */
  updateNextGameVertexId: number;
  /** The game vertex it walks from, `m_tPrevGraphID`. */
  updatePreviousGameVertexId: number;
  updateDistanceFromPoint: number | null;
  updateDistanceToPoint: number | null;
};

/**
 * A mutant, `CSE_ALifeMonsterBase`: a monster with a physics skeleton, which every monster class Call of Pripyat
 * registers is, the psy dog's phantom included. Its update packet is the monster's; the skeleton adds none.
 */
export type AlifeObjectMonsterBase = {
  base: AlifeObjectMonsterAbstract;
  skeleton: AlifeObjectSkeleton;
  /** The object the monster is bound to, `m_spec_object_id`: the psy dog a phantom belongs to; `u16::MAX` for none. */
  specObjectId: number;
};

export type AlifeObjectMotion = {
  motionName: string;
};

/**
 * A squad's server object, `CSE_ALifeOnlineOfflineGroup`: the alife object, then a `u32` count of its members' object
 * ids. Its update packet is empty.
 */
export type AlifeObjectOnlineOfflineGroup = {
  base: AlifeObjectDynamic;
  /** The members' object ids, `m_members`, which the game assigns when it creates them. */
  members: Array<number>;
};

/**
 * A physics skeleton object, `CSE_ALifePHSkeletonObject`: a visual with a physics skeleton, such as a corpse left as
 * a prop. Its update packet is empty.
 */
export type AlifeObjectPhSkeletonObject = {
  base: AlifeObjectDynamicVisual;
  skeleton: AlifeObjectSkeleton;
};

export type AlifeObjectPhysic = {
  base: AlifeObjectDynamicVisual;
  skeleton: AlifeObjectSkeleton;
  physicType: number;
  mass: number | null;
  fixedBones: string;
  /**
   * The physics state the update packet carries. `CPhysicObject::net_Import` drops it for a local object, so a single
   * player game never applies it, which is why vanilla's are often left uninitialised.
   */
  updateState: PhysicsNetState | null;
};

export type AlifeObjectShape = {
  base: AlifeObjectAbstract;
  shape: Array<Shape>;
};

export type AlifeObjectSkeleton = {
  /** The visual's startup animation (`CSE_Visual::startup_animation`), which `CSE_PHSkeleton::STATE_Read` reads first. */
  name: string;
  flags: number;
  sourceId: number;
};

/**
 * `CSE_SmartCover`'s own fields. Below spawn version 128 the packet has no `can_fire`, which the engine then sets to
 * the combat cover flag (`xrServer_Objects_Alife_Smartcovers.cpp:110`).
 */
export type AlifeObjectSmartCover = {
  base: AlifeObjectDynamic;
  shape: Array<Shape>;
  description: string;
  holdPositionTime: number | null;
  enterMinEnemyDistance: number | null;
  exitMinEnemyDistance: number | null;
  isCombatCover: number;
  canFire: number;
};

export type AlifeObjectSpaceRestrictor = {
  base: AlifeObjectAbstract;
  shape: Array<Shape>;
  restrictorType: number;
};

/**
 * A stationary machine gun, `CSE_ALifeStationaryMgun`: a visual whose update packet carries whether it fires and
 * where it aims. The engine leaves both uninitialised in a new gun, so the aim keeps the bits it is stored in.
 */
export type AlifeObjectStationaryMgun = {
  base: AlifeObjectDynamicVisual;
  /** Whether it fires, `m_bWorking`, one byte. */
  updateWorking: number;
  /** The direction it aims at its enemy in, `m_destEnemyDir`. */
  updateEnemyDirection: [RawFloat, RawFloat, RawFloat];
};

export type AlifeObjectTorridZone = {
  base: AlifeObjectCustomZone;
  motion: AlifeObjectMotion;
  lastSpawnTime: LastSpawnTime;
};

export type AlifeObjectTrader = {
  base: AlifeObjectDynamicVisual;
  trader: AlifeObjectTraderAbstract;
};

/**
 * `CSE_ALifeTraderAbstract`, the trading part of a trader, a human and the actor. Up to spawn version 124 the packet
 * stores neither of its corpse's access fields, which then hold the constructor's: open to taking, not closed
 * (`xrServer_Objects_ALife_Monsters.cpp:138-139`).
 */
export type AlifeObjectTraderAbstract = {
  money: number;
  specificCharacter: string;
  traderFlags: number;
  characterProfile: string;
  communityIndex: number;
  rank: number;
  reputation: number;
  characterName: string;
  deadBodyCanTake: number;
  deadBodyClosed: number;
};

export type AlifeObjectVisual = {
  visualName: string;
  visualFlags: number;
};

/**
 * `se_smart_cover`, the script class over the server smart cover. Below script version 9 its packet has no last
 * description or loopholes, which then hold the script constructor's empty values (`se_smart_cover.script`'s
 * `__init`); Call of Pripyat's script fills them from its smart cover descriptions, script config no packet carries.
 */
export type AlifeSmartCover = {
  base: AlifeObjectSmartCover;
  lastDescription: string;
  loopholes: Array<AlifeSmartCoverLoophole>;
};

export type AlifeSmartCoverLoophole = {
  name: string;
  enabled: number;
};

export type AlifeSmartTerrain = {
  base: AlifeSmartZone;
  /**
   * The script class's own fields, `se_smart_terrain` in vanilla and `SmartTerrain` in xrf-engine: list counts and
   * presence flags, each zero in a compiled spawn because the simulation has not run.
   */
  arrivingObjectsCount: number;
  objectJobDescriptorsCount: number;
  deadObjectsInfosCount: number;
  smartTerrainActorControl: number;
  respawnPoint: number;
  stayingObjectsCount: number;
};

export type AlifeSmartZone = {
  base: AlifeObjectSpaceRestrictor;
};

/**
 * A squad as its script class saves one, `sim_squad_scripted` in vanilla and `Squad` in xrf-engine: both follow the
 * group's fields with four `tostring` strings closed by a save marker - its target, its respawn point and the section
 * that point spawns, and its smart terrain - and add nothing to the update packet.
 */
export type AlifeSquad = {
  base: AlifeObjectOnlineOfflineGroup;
  /** The object the squad heads for, `current_target_id`. */
  currentTargetId: number | null;
  /** The smart terrain that respawns it, `respawn_point_id`. */
  respawnPointId: number | null;
  /** The respawn section it was spawned from, `respawn_point_prop_section`. */
  respawnPointSection: string | null;
  /** The smart terrain it is assigned to, `smart_id` (`assignedTerrainId` in xrf-engine). */
  assignedTerrainId: number | null;
};

/**
 * A stalker as its script class `se_stalker` saves one: vanilla's `se_stalker.script` and xrf-engine's `Stalker.ts`
 * both follow the server class's spawn fields with the offline state and whether the corpse's loot was dropped,
 * unmarked, and add nothing to the update packet.
 *
 * The tail mirrors `STATE_Write` in xrf-engine's `src/engine/core/objects/creature/Stalker.ts`, whose own test pins
 * its write order, so a change to either side changes the other in the same work.
 */
export type AlifeStalker = {
  base: AlifeObjectHumanStalker;
  offlineState: OfflineObjectState;
  /**
   * Whether the death manager already dropped the corpse's loot, `w_bool` of `death_droped` (`isCorpseLootDropped`
   * in xrf-engine); zero for a new object.
   */
  lootDropped: number;
};

/**
 * A multiplayer team's base, `CSE_ALifeTeamBaseZone` (`zone_team_base`, class `Z_TEAMBS`): a space restrictor and the
 * team it belongs to, which the team modes respawn and score at.
 */
export type AlifeTeamBaseZone = {
  base: AlifeObjectSpaceRestrictor;
  /** `m_team`, which the Level Editor offers from 0 to 16. */
  team: number;
};

export type AlifeZoneVisual = {
  base: AlifeObjectAnomalyZone;
  visual: AlifeObjectVisual;
  /** The animation the zone idles in (`CSE_Visual::startup_animation`). */
  idleAnimation: string;
  attackAnimation: string;
  lastSpawnTime: LastSpawnTime;
};

export type ArtefactSpawnPoint = {
  position: Vector3d;
  levelVertexId: number;
  distance: number | null;
};

/**
 * The combat state a Clear Sky smart terrain saves, `CCombat_manager:save` with its `CCover_manager:save`
 * (`gamedata-cs/scripts/sim_combat.script`, `cover_manager.script`), each closed by its own save marker.
 *
 * The squads in combat, the squads forced online, the cover table and the time the actor was last seen are records
 * only a running simulation fills; a spawn holds none of them, so their counts and flags are read as empty and any
 * other value is refused rather than read.
 */
export type ClearSkyCombatManager = {
  /** `actor_defence_come`, a `w_bool`. */
  actorDefenceCome: number;
  /** `combat_quest` as written; the constructor's is the text `nil`. */
  combatQuest: string;
  /** The task's id, `u16::MAX` for none. */
  taskId: number;
  /** `see_actor_enemy` as the script writes it, `tostring` of a value; `None` for `nil`. */
  seeActorEnemy: string | null;
  /** `force_online`, a `w_bool`. */
  forceOnline: number;
  /** The cover manager's `is_valid`, a `w_bool`. */
  coverIsValid: number;
};

/**
 * The crow shooting minigame's state, `CMGCrowKiller:save` (`gamedata-cs/scripts/minigame_crowkiller.script`): the
 * parameters the minigames manager keeps for it, then the game's own counters, closed by a save marker.
 */
export type ClearSkyCrowKiller = {
  highscore: number;
  timer: number;
  /** The parameter `win`, a `w_bool`. */
  win: number;
  crowsToKill: Array<number>;
  moneyMultiplier: number;
  championMultiplier: number;
  selected: number;
  gameType: string;
  /** The game's own `high_score`. */
  currentHighScore: number;
  /** The game's own `timer`. */
  currentTimer: number;
  timeOut: number;
  killedCounter: number;
  /** The game's own `win`, a `w_bool`. */
  currentWin: number;
};

/** Every `type` the `ClearSkyJobOnline` union is told apart by, so a switch or a comparison names one. */
export enum EClearSkyJobOnline {
  /** `true`, written as 0. */
  ONLINE = "Online",
  /** `false`, written as 1. */
  OFFLINE = "Offline",
  /** `nil`, written as 2: the job does not decide. */
  UNSET = "Unset",
  /** A condition list deciding it, written as 3 and the list's text. */
  CONDLIST = "Condlist",
}

/**
 * Whether a Clear Sky stalker or monster is kept online or offline by its job, `job_online` of `se_stalker` and
 * `se_monster`: a byte, followed by the condition list when the byte is 3. The scripts read any byte past 2 as a
 * condition list and write 3, so only 0 to 3 are accepted. A spawn holds `Unset`.
 */
export type ClearSkyJobOnline =
  /** `true`, written as 0. */
  | { type: "Online" }
  /** `false`, written as 1. */
  | { type: "Offline" }
  /** `nil`, written as 2: the job does not decide. */
  | { type: "Unset" }
  /** A condition list deciding it, written as 3 and the list's text. */
  | { type: "Condlist"; value: string };

/**
 * One minigame of Clear Sky's minigames manager, as `CMinigames:save` writes it
 * (`gamedata-cs/scripts/minigames_manager.script`): its name, its game's profile name, its state, then the game's own
 * state.
 */
export type ClearSkyMinigame = {
  name: string;
  /** What the manager holds it as: `valiable` (sic), `enabled`, `started` or `running`. */
  state: string;
  game: ClearSkyMinigameGame;
};

/**
 * The game a Clear Sky minigame runs, by the profile name the minigames manager saves before its state: the two
 * `CMinigames:load` builds. Any other profile leaves the game without one, so it is refused.
 */
export type ClearSkyMinigameGame =
  | ({
      type: "CrowKiller";
    } & ClearSkyCrowKiller)
  | ({
      type: "Shooting";
    } & ClearSkyShooting);

/**
 * An id generator of Clear Sky's scripts, `object_collection:save` (`gamedata-cs/scripts/object_collection.script`):
 * how many ids are given and the last one, then the freed and the given ids, closed by a save marker. A spawn holds a
 * generator that gave none, so the two id lists are read as empty and any other count is refused rather than read.
 */
export type ClearSkyObjectCollection = {
  /** `m_count`. */
  count: number;
  /** `m_last_id`. */
  lastId: number;
};

/**
 * The shooting range minigame's state, `CMGShooting:save` (`gamedata-cs/scripts/minigame_shooting.script`): the
 * parameters the minigames manager keeps for it, the inventory it took from the actor and the game in progress, closed
 * by a save marker.
 *
 * The targets, the inventory taken and the game in progress are what playing fills; a spawn holds no targets, nothing
 * taken and the game `nil`, so those are read as such and any other value is refused rather than read.
 */
export type ClearSkyShooting = {
  gameType: string;
  wpnType: string;
  /** The patrol paths, each `tostring` of a name; `None` for `nil`. */
  standWay: string | null;
  lookWay: string | null;
  standWayBack: string | null;
  lookWayBack: string | null;
  /** The target object's name; `None` for `nil`. */
  objName: string | null;
  win: ClearSkyShootingWin;
  distance: number;
  ammo: number;
  targetCounter: number;
  prevTime: number;
};

/** Every `type` the `ClearSkyShootingWin` union is told apart by, so a switch or a comparison names one. */
export enum EClearSkyShootingWin {
  FLAG = "Flag",
  COUNT = "Count",
}

/**
 * The shooting minigame's `win` parameter, a boolean or a count by the game played: a kind byte, 0 for a `w_bool` and
 * 1 for a `u16`. The script reads any other kind as a count and writes 1, so only 0 and 1 are accepted.
 */
export type ClearSkyShootingWin = { type: "Flag"; value: number } | { type: "Count"; value: number };

/** todo: Add script to parse system ltx and read all the data from ltx/txt file instead. */
export enum EClsId {
  AI_CROW = "AiCrow",
  AI_GRAPH = "AiGraph",
  AI_PHANT = "AiPhant",
  AI_STL_S = "AiStlS",
  AI_TRD_S = "AiTrdS",
  AMMO_S = "AmmoS",
  ARTEFACT = "Artefact",
  C_HLCP_S = "CHlcpS",
  D_FLARE = "DFlare",
  D_PDA = "DPda",
  DET_ADVA = "DetAdva",
  DET_ELIT = "DetElit",
  DET_SIMP = "DetSimp",
  DET_SCIE = "DetScie",
  E_HLMET = "EHlmet",
  E_STLK = "EStlk",
  GF1_S = "GF1S",
  G_FAKE = "GFake",
  G_RGD5_S = "GRgd5S",
  G_RPG7 = "GRpg7",
  II_ATTCH = "IIAttch",
  II_BOLT = "IIBolt",
  II_BTTCH = "IIBttch",
  II_DOC = "IIDoc",
  LVL_CHNG = "LvlChng",
  NW_ATTCH = "NwAttch",
  O_BRKBL = "OBrkbl",
  O_CLMBL = "OClmbl",
  O_DSTR_S = "ODstrS",
  O_PHYS_S = "OPhysS",
  O_SEARCH = "OSearch",
  ON_OFF_S = "OnOffS",
  P_SKELET = "PSkelet",
  S_ACTOR = "SActor",
  S_EXPLO = "SExplo",
  /** `SFACTION`, Clear Sky's `se_sim_faction`. */
  S_FACTION = "SFaction",
  S_FOOD = "SFood",
  S_INV_BOX = "SInvBox",
  SM209 = "SM209",
  SOG7_B = "SOG7B",
  S_PDA = "SPda",
  S_VOG25 = "SVog25",
  SCRIPT_ZN = "ScriptZn",
  SCRPT_ART = "ScrptArt",
  SCRPT_CAR = "ScrptCar",
  SCRPT_OBJ = "ScrptObj",
  SM_BLOOD = "SmBlood",
  SM_BOAR_W = "SmBoarW",
  SM_BURER = "SmBurer",
  SM_CHIMS = "SmChims",
  SM_CONTR = "SmContr",
  SM_DOG_F = "SmDogF",
  SM_DOG_P = "SmDogP",
  SM_DOG_S = "SmDogS",
  SM_FLESH = "SmFlesh",
  SM_GIANT = "SmGiant",
  SM_P_DOG = "SmPDog",
  SM_POLTR = "SmPoltr",
  SM_SNORK = "SmSnork",
  SM_TUSHK = "SmTushk",
  SMRT_CS = "SmrtCS",
  SMRT_TRRN = "SmrtTrrn",
  SO_H_LAMP = "SoHLamp",
  SPC_RS_S = "SpcRsS",
  TORCH_S = "TorchS",
  WSTM_GUN = "WSTMGun",
  WP_AK74 = "WpAk74",
  WP_ASH_TG = "WpAshTG",
  WP_BM16 = "WpBM16",
  WP_BINOC = "WpBinoc",
  WP_G_LAUN = "WpGLaun",
  WP_GROZA = "WpGroza",
  WP_HPSA = "WpHPSA",
  WP_KNIFE = "WpKnife",
  WP_LR300 = "WpLR300",
  WP_PM = "WpPM",
  WP_RG6 = "WpRG6",
  WP_RPG7 = "WpRPG7",
  WP_SVD = "WpSVD",
  WP_SVU = "WpSVU",
  WP_SCOPE = "WpScope",
  WP_SILEN = "WpSilen",
  WP_VAL = "WpVAL",
  Z_ACID_F = "ZAcidF",
  Z_AMEBA = "ZAmeba",
  ZB_FUZZ = "ZBFuzz",
  ZC_FIRE = "ZCFire",
  Z_DEAD = "ZDead",
  Z_GALANT = "ZGalant",
  Z_MBALD = "ZMbald",
  Z_MINCER = "ZMincer",
  Z_NO_GRAV = "ZNoGrav",
  Z_RADIO = "ZRadio",
  Z_RUSTY_H = "ZRustyH",
  Z_TEAM_BS = "ZTeamBs",
  Z_TORRID = "ZTorrid",
  ZS_AMEBA = "ZsAmeba",
  ZS_B_FUZZ = "ZsBFuzz",
  ZS_GALAN = "ZsGalan",
  ZS_M_BALD = "ZsMBald",
  ZS_MINCE = "ZsMince",
  ZS_N_GRAV = "ZsNGrav",
  ZS_RADIO = "ZsRadio",
  ZS_TORRD = "ZsTorrd",
}

/** Every `EClsId` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type ClsId = `${EClsId}`;

export type GraphCrossTable = {
  version: number;
  nodesCount: number;
  verticesCount: number;
  levelGuid: string;
  gameGuid: string;
};

export type GraphEdge = {
  gameVertexId: number;
  distance: number | null;
};

export type GraphHeader = {
  version: number;
  verticesCount: number;
  edgesCount: number;
  pointsCount: number;
  guid: string;
  levelsCount: number;
};

/** `GameGraph::SLevel::load` in xray codebase. */
export type GraphLevel = {
  name: string;
  offset: Vector3d;
  id: number;
  section: string;
  guid: string;
};

export type GraphLevelPoint = {
  position: Vector3d;
  levelVertexId: number;
  distance: number | null;
};

export type GraphVertex = {
  levelPoint: Vector3d;
  gamePoint: Vector3d;
  levelId: number;
  levelVertexId: number;
  vertexType: U32Bytes;
  edgesOffset: number;
  levelPointsOffset: number;
  edgesCount: number;
  levelPointsCount: number;
};

/** Every `type` the `LastSpawnTime` union is told apart by, so a switch or a comparison names one. */
export enum ELastSpawnTime {
  /** The script class wrote nothing past the engine payload. */
  ABSENT = "Absent",
  /** The script class wrote its flag with no time behind it. */
  UNSET = "Unset",
  /** The script class wrote its flag and a time. */
  SET = "Set",
}

/** Trailing spawn time a fork's script class appends after the engine server class payload. */
export type LastSpawnTime =
  /** The script class wrote nothing past the engine payload. */
  | { type: "Absent" }
  /** The script class wrote its flag with no time behind it. */
  | { type: "Unset" }
  /** The script class wrote its flag and a time. */
  | { type: "Set"; value: Time };

/**
 * What `se_stalker` and `se_monster` keep of an object while it is offline, written after their server class's fields
 * as two strings, each `tostring` of a value or of `nil`: the level vertex it last stood on and the logic section it
 * was running. Neither class closes them with a save marker. A compiled spawn holds `nil` for both, which is what a
 * new object starts with.
 */
export type OfflineObjectState = {
  levelVertexId: number | null;
  activeSection: string | null;
};

/**
 * Patrols list is represented by list of samples containing patrol chunk.
 * 0...N, where N is chunk.
 *
 * `CPatrolPathStorage::load`, `CPatrolPath::load_raw` in xray codebase.
 *
 * Patrol chunk has the following structure:
 * 0 - metadata
 *   - name
 * 1 - data
 *     0 - points count
 *     1 - patrol points
 *     2 - patrol points links
 */
export type Patrol = {
  name: string;
  points: Array<PatrolPoint>;
  links: Array<PatrolLink>;
};

export type PatrolLink = {
  index: number;
  links: Array<[number, number | null]>;
};

/** `CPatrolPoint::load_raw`, `CPatrolPoint::load` in xray codebase. */
export type PatrolPoint = {
  name: string;
  position: Vector3d;
  flags: number;
  levelVertexId: number;
  gameVertexId: number;
};

/**
 * The physics state an inventory item and a physic object carry in their update packets, `SPHNetState` behind the
 * byte `CSE_ALifeInventoryItem::UPDATE_Write` and `CSE_ALifeObjectPhysic::UPDATE_Write` open it with.
 *
 * That byte packs the item count into its low five bits and, above them, a mask: whether the body is enabled, and
 * whether each velocity is left out as zero. A zero byte is a state that is not there at all, so an object without one
 * is `None` beside its other update fields rather than a state with dead fields; but for a physic object's state,
 * which closes its packet, a zero byte with bytes behind it is a state packed to zero (`read_closing`). The
 * SDK writes some states without initialising them, so each float keeps the bits it is stored in.
 */
export type PhysicsNetState = {
  /** The synchronised items, `m_u8NumItems`, at most the 31 its five bits hold. */
  itemCount: number;
  enabled: boolean;
  force: [RawFloat, RawFloat, RawFloat];
  torque: [RawFloat, RawFloat, RawFloat];
  /** Where the body stands; an item's `UPDATE_Read` moves the object here (`o_Position`). */
  position: [RawFloat, RawFloat, RawFloat];
  /** The orientation quaternion as x, y, z and w. */
  rotation: [RawFloat, RawFloat, RawFloat, RawFloat];
  /** `None` where the mask leaves the angular velocity out as zero. */
  angularVelocity: [RawFloat, RawFloat, RawFloat] | null;
  /** `None` where the mask leaves the linear velocity out as zero. */
  linearVelocity: [RawFloat, RawFloat, RawFloat] | null;
  /** The byte closing the state: zero freezes the object and anything else leaves it awake. */
  awake: number;
};

/**
 * A float kept as the bits it is stored in, for state the SDK writes without initialising: a vanilla spawn's
 * physics states hold NaNs with payloads, and whole states of `0xffffffff`, which no decimal brings back.
 *
 * A finite value travels as its number, in LTX and JSON alike, and anything else as its bits in hex (`0xffffffff`).
 */
export type RawFloat = number | string;

/** A vector as `NET_Packet::w_sdir` writes it: its direction compressed into a `u16` (`pvCompress`), then its length. */
export type ScaledDirection = {
  /** The unit direction as `pvCompress` packs it. */
  direction: number;
  magnitude: number | null;
};

/** One shape of an object's volume, `CShapeData::shape_def`, in the object's own space. */
export type Shape =
  | ({ Sphere: Sphere } & { Box?: never })
  /**
   * The unit cube centred on the origin, placed by the transform: its axes are the box's sides and its
   * translation the box's centre.
   */
  | ({ Box: Affine3d } & { Sphere?: never });

/**
 * ALife spawns chunk has the following structure:
 * 0 - count
 * 1 - objects
 * 2 - edges
 */
export type SpawnALifeSpawnsChunk = {
  objects: Array<AlifeObject>;
};

/**
 * Artefacts spawns samples.
 * Is single plain chunk with nodes list in it.
 */
export type SpawnArtefactSpawnsChunk = {
  nodes: Array<ArtefactSpawnPoint>;
};

/** `GameGraph::CHeader::load`, `GameGraph::SLevel::load`, `CGameGraph::Initialize` */
export type SpawnGraphsChunk = {
  header: GraphHeader;
  levels: Array<GraphLevel>;
  vertices: Array<GraphVertex>;
  edges: Array<GraphEdge>;
  points: Array<GraphLevelPoint>;
  crossTables: Array<GraphCrossTable>;
};

export type SpawnHeaderChunk = {
  version: number;
  guid: string;
  graphGuid: string;
  objectsCount: number;
  levelsCount: number;
};

/**
 * What every server entity's spawn packet opens with, `CSE_Abstract::Spawn_Write` (`xrServer_Object_Base.cpp:156`),
 * up to the size it patches in once its class's own fields, the state, are written.
 */
export type SpawnPacketHeader = {
  id: number;
  /** `M_SPAWN`, which the engine asserts on before reading anything else. */
  netAction: number;
  /** The config section the entity is built from. */
  section: string;
  /** The name the level gives this one, `s_name_replace`. */
  name: string;
  scriptGameId: number;
  scriptRp: number;
  position: Vector3d;
  direction: Vector3d;
  respawnTime: number;
  parentId: number;
  phantomId: number;
  /** `s_flags`; the spawn version is stored only while `M_SPAWN_VERSION` is set. */
  scriptFlags: number;
  version: number;
  gameType: number;
  scriptVersion: number;
  /**
   * The client object's saved state (`CSE_Abstract::client_data`), raw bytes carried as stored, which fields cannot
   * hold.
   */
  clientData: Array<number>;
  spawnId: number;
};

/** `CPatrolPathStorage::load` in xray engine. */
export type SpawnPatrolsChunk = {
  patrols: Array<Patrol>;
};

export type Time = {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
  millis: number;
};

export type U32Bytes = [number, number, number, number];
