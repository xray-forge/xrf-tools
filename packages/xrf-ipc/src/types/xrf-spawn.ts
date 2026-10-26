// Auto-generated rust bindings. Do not edit it manually.

import { Vector3d } from "#/types/xrf-math";

export type AlifeActor = {
  base: AlifeObjectActor;
  /** The script class's own field, `se_actor` in vanilla. */
  startPositionFilled: number;
  /**
   * The update packet, carried as stored. The engine reads it over the spawn packet, so the actor starts where it
   * says.
   */
  updateData: Array<number>;
};

export type AlifeAnomalousZone = {
  base: AlifeObjectAnomalyZone;
  lastSpawnTime: LastSpawnTime;
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

/** Generic abstract ALife object base. */
export type AlifeObject = {
  id: number;
  netAction: number;
  section: string;
  clsid: ClsId;
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
  /** The client object's saved state (`CSE_Abstract::client_data`), carried as stored. */
  clientData: Array<number>;
  spawnId: number;
  /**
   * The server class payload, its fields from both packets. The engine applies the update packet over the spawn
   * packet when it creates the object, so where both carry a field the update packet's is the one the game uses.
   */
  inherited: AlifeObjectInherited;
};

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

export type AlifeObjectActor = {
  base: AlifeObjectCreature;
  trader: AlifeObjectTraderAbstract;
  skeleton: AlifeObjectSkeleton;
  holderId: number;
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

export type AlifeObjectClimable = {
  base: AlifeObjectShape;
  gameMaterial: string;
};

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

export type AlifeObjectInherited =
  | ({
      type: "SeActor";
    } & AlifeActor)
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
    } & AlifeObjectItemWeaponMagazinedWgl);

export type AlifeObjectInventoryBox = {
  base: AlifeObjectDynamicVisual;
  canTake: number;
  isClosed: number;
  tip: string;
};

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
 * A weapon, `CSE_ALifeItemWeapon`. Its update packet repeats the rounds loaded, weapon state, addons and ammo type the
 * spawn packet stores, and the engine reads them over the spawn packet's, so those are what the game starts with.
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

export type AlifeObjectMotion = {
  motionName: string;
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

export type AlifeObjectTorridZone = {
  base: AlifeObjectCustomZone;
  motion: AlifeObjectMotion;
  lastSpawnTime: LastSpawnTime;
};

export type AlifeObjectTrader = {
  base: AlifeObjectDynamicVisual;
  trader: AlifeObjectTraderAbstract;
};

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

/** Represents script extension of base server smart cover class. */
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

/** todo: Add script to parse system ltx and read all the data from ltx/txt file instead. */
export enum EClsId {
  AI_CROW = "AiCrow",
  AI_FLE_G = "AiFleG",
  AI_GRAPH = "AiGraph",
  AI_PHANT = "AiPhant",
  AI_RAT = "AiRat",
  AI_RAT_G = "AiRatG",
  AI_SP_GRP = "AiSpGrp",
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
  P_SKELET = "PSkelet",
  S_ACTOR = "SActor",
  S_EXPLO = "SExplo",
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
  SPECT = "Spect",
  TORCH_S = "TorchS",
  W_MOUNTD = "WMountd",
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
 * is `None` beside its other update fields rather than a state with dead fields. The SDK writes some states without
 * initialising them, so each float keeps the bits it is stored in.
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

/** Shape enumeration stored in objects descriptors. */
export type Shape =
  | ({ Sphere: [Vector3d, number | null] } & { Box?: never })
  | ({ Box: [Vector3d, Vector3d, Vector3d, Vector3d] } & { Sphere?: never });

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
