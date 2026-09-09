import { EApplicationId } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

/**
 * Backend job identifiers used for registration, adoption, and notification attribution.
 */
export enum EJobKind {
  ARCHIVES_EXTRACT = "archives.extract",
  ARCHIVES_COMPARE = "archives.compare",
  ARCHIVES_PACK = "archives.pack",
  ARCHIVES_PATCH = "archives.patch",
  ARCHIVES_UNPACK = "archives.unpack",
  CONFIGS_CHECK_FORMAT = "configs.check-format",
  CONFIGS_FORMAT = "configs.format",
  CONFIGS_VERIFY = "configs.verify",
  SPAWN_PACK = "spawn.pack",
  SPAWN_UNPACK = "spawn.unpack",
  SPRITE_EQUIPMENT_PACK = "sprite-equipment.pack",
  GAMEDATA_VERIFY = "gamedata.verify",
  TEXTURES_BUILD = "textures.build",
  TEXTURES_COMPARE_ENCODINGS = "textures.compare-encodings",
  TEXTURES_MAKE_BUMP = "textures.make-bump",
  TEXTURES_SAVE = "textures.save",
  TRANSLATIONS_BUILD = "translations.build",
  TRANSLATIONS_CHECK_FORMAT = "translations.check-format",
  TRANSLATIONS_FORMAT = "translations.format",
  TRANSLATIONS_PARSE = "translations.parse",
  TRANSLATIONS_VERIFY = "translations.verify",
}

/**
 * Display metadata for a backend job kind.
 */
export interface IJobKindDescriptor {
  kind: EJobKind;
  /**
   * Application credited in job notifications.
   */
  source: EApplicationId;
  /**
   * Display label for the job kind.
   */
  label: string;
}

/**
 * Job metadata available independently of application lifetimes.
 */
export const JOB_KINDS: Record<EJobKind, IJobKindDescriptor> = {
  [EJobKind.ARCHIVES_COMPARE]: {
    kind: EJobKind.ARCHIVES_COMPARE,
    source: EApplicationId.ARCHIVES_PATCHER,
    label: "Archive comparison",
  },
  [EJobKind.ARCHIVES_PATCH]: {
    kind: EJobKind.ARCHIVES_PATCH,
    source: EApplicationId.ARCHIVES_PATCHER,
    label: "Archive patching",
  },
  [EJobKind.SPAWN_PACK]: {
    kind: EJobKind.SPAWN_PACK,
    source: EApplicationId.SPAWN_PACKER,
    label: "Spawn packing",
  },
  [EJobKind.SPAWN_UNPACK]: {
    kind: EJobKind.SPAWN_UNPACK,
    source: EApplicationId.SPAWN_UNPACKER,
    label: "Spawn unpacking",
  },
  [EJobKind.ARCHIVES_EXTRACT]: {
    kind: EJobKind.ARCHIVES_EXTRACT,
    source: EApplicationId.ARCHIVES_EXPLORER,
    label: "Archive extraction",
  },
  [EJobKind.ARCHIVES_PACK]: {
    kind: EJobKind.ARCHIVES_PACK,
    source: EApplicationId.ARCHIVES_PACKER,
    label: "Archive packing",
  },
  [EJobKind.ARCHIVES_UNPACK]: {
    kind: EJobKind.ARCHIVES_UNPACK,
    source: EApplicationId.ARCHIVES_UNPACKER,
    label: "Archive unpacking",
  },
  [EJobKind.CONFIGS_CHECK_FORMAT]: {
    kind: EJobKind.CONFIGS_CHECK_FORMAT,
    source: EApplicationId.CONFIGS_FORMATTER,
    label: "Config format check",
  },
  [EJobKind.CONFIGS_FORMAT]: {
    kind: EJobKind.CONFIGS_FORMAT,
    source: EApplicationId.CONFIGS_FORMATTER,
    label: "Config formatting",
  },
  [EJobKind.CONFIGS_VERIFY]: {
    kind: EJobKind.CONFIGS_VERIFY,
    source: EApplicationId.CONFIGS_VERIFIER,
    label: "Config verification",
  },
  [EJobKind.TRANSLATIONS_BUILD]: {
    kind: EJobKind.TRANSLATIONS_BUILD,
    source: EApplicationId.TRANSLATIONS_BUILDER,
    label: "Translation build",
  },
  [EJobKind.TRANSLATIONS_CHECK_FORMAT]: {
    kind: EJobKind.TRANSLATIONS_CHECK_FORMAT,
    source: EApplicationId.TRANSLATIONS_FORMATTER,
    label: "Translation format check",
  },
  [EJobKind.TRANSLATIONS_FORMAT]: {
    kind: EJobKind.TRANSLATIONS_FORMAT,
    source: EApplicationId.TRANSLATIONS_FORMATTER,
    label: "Translation formatting",
  },
  [EJobKind.TRANSLATIONS_PARSE]: {
    kind: EJobKind.TRANSLATIONS_PARSE,
    source: EApplicationId.TRANSLATIONS_PARSER,
    label: "Translation import",
  },
  [EJobKind.TRANSLATIONS_VERIFY]: {
    kind: EJobKind.TRANSLATIONS_VERIFY,
    source: EApplicationId.TRANSLATIONS_VERIFIER,
    label: "Translation check",
  },
  [EJobKind.SPRITE_EQUIPMENT_PACK]: {
    kind: EJobKind.SPRITE_EQUIPMENT_PACK,
    source: EApplicationId.SPRITE_EQUIPMENT_PACKER,
    label: "Equipment sprite packing",
  },
  [EJobKind.GAMEDATA_VERIFY]: {
    kind: EJobKind.GAMEDATA_VERIFY,
    source: EApplicationId.GAMEDATA_VERIFIER,
    label: "Gamedata verification",
  },
  [EJobKind.TEXTURES_SAVE]: {
    kind: EJobKind.TEXTURES_SAVE,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Texture save",
  },
  [EJobKind.TEXTURES_BUILD]: {
    kind: EJobKind.TEXTURES_BUILD,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Texture build",
  },
  [EJobKind.TEXTURES_MAKE_BUMP]: {
    kind: EJobKind.TEXTURES_MAKE_BUMP,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Bump pair generation",
  },
  [EJobKind.TEXTURES_COMPARE_ENCODINGS]: {
    kind: EJobKind.TEXTURES_COMPARE_ENCODINGS,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Texture format comparison",
  },
};

/**
 * Looks up display metadata for a backend job kind.
 *
 * @param kind - Backend job identifier.
 * @returns Metadata, or `null` for an unknown kind.
 */
export function findJobKind(kind: string): Nullable<IJobKindDescriptor> {
  return JOB_KINDS[kind as EJobKind] ?? null;
}
