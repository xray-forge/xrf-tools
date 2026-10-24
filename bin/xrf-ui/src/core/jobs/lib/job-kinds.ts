import {
  ARCHIVES_JOB_COMPARE,
  ARCHIVES_JOB_EXTRACT,
  ARCHIVES_JOB_PACK,
  ARCHIVES_JOB_PATCH,
  ARCHIVES_JOB_UNPACK,
} from "@xrf/ipc/types/xrf-app-archives";
import { CONFIGS_JOB_CHECK_FORMAT, CONFIGS_JOB_FORMAT, CONFIGS_JOB_VERIFY } from "@xrf/ipc/types/xrf-app-configs";
import { JobKind } from "@xrf/ipc/types/xrf-app-core";
import { GAMEDATA_JOB_VERIFY } from "@xrf/ipc/types/xrf-app-gamedata";
import { LEVELS_JOB_CAPTURE_CUBE, LEVELS_JOB_CAPTURE_MAP } from "@xrf/ipc/types/xrf-app-levels";
import { SPAWN_JOB_PACK, SPAWN_JOB_UNPACK } from "@xrf/ipc/types/xrf-app-spawn";
import { SPRITE_EQUIPMENT_JOB_PACK } from "@xrf/ipc/types/xrf-app-sprite-equipment";
import {
  TEXTURES_JOB_BUILD,
  TEXTURES_JOB_COMPARE_ENCODINGS,
  TEXTURES_JOB_MAKE_BUMP,
  TEXTURES_JOB_SAVE,
} from "@xrf/ipc/types/xrf-app-textures";
import {
  TRANSLATIONS_JOB_BUILD,
  TRANSLATIONS_JOB_CHECK_FORMAT,
  TRANSLATIONS_JOB_FORMAT,
  TRANSLATIONS_JOB_PARSE,
  TRANSLATIONS_JOB_VERIFY,
} from "@xrf/ipc/types/xrf-app-translations";
import { Nullable } from "@xrf/types";

import { EApplicationId } from "@/core/routing/application";

/**
 * Display metadata for a backend job kind.
 */
export interface IJobKindDescriptor {
  kind: JobKind;
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
 * Job metadata available independently of application lifetimes, keyed by the generated job kind constants; a test
 * holds it to every kind the backend declares.
 */
export const JOB_KINDS: Readonly<Record<JobKind, IJobKindDescriptor>> = {
  [ARCHIVES_JOB_COMPARE]: {
    kind: ARCHIVES_JOB_COMPARE,
    source: EApplicationId.ARCHIVES_PATCHER,
    label: "Archive comparison",
  },
  [ARCHIVES_JOB_PATCH]: {
    kind: ARCHIVES_JOB_PATCH,
    source: EApplicationId.ARCHIVES_PATCHER,
    label: "Archive patching",
  },
  [SPAWN_JOB_PACK]: {
    kind: SPAWN_JOB_PACK,
    source: EApplicationId.SPAWN_PACKER,
    label: "Spawn packing",
  },
  [SPAWN_JOB_UNPACK]: {
    kind: SPAWN_JOB_UNPACK,
    source: EApplicationId.SPAWN_UNPACKER,
    label: "Spawn unpacking",
  },
  [ARCHIVES_JOB_EXTRACT]: {
    kind: ARCHIVES_JOB_EXTRACT,
    source: EApplicationId.ARCHIVES_EXPLORER,
    label: "Archive extraction",
  },
  [ARCHIVES_JOB_PACK]: {
    kind: ARCHIVES_JOB_PACK,
    source: EApplicationId.ARCHIVES_PACKER,
    label: "Archive packing",
  },
  [ARCHIVES_JOB_UNPACK]: {
    kind: ARCHIVES_JOB_UNPACK,
    source: EApplicationId.ARCHIVES_UNPACKER,
    label: "Archive unpacking",
  },
  [CONFIGS_JOB_CHECK_FORMAT]: {
    kind: CONFIGS_JOB_CHECK_FORMAT,
    source: EApplicationId.CONFIGS_FORMATTER,
    label: "Config format check",
  },
  [CONFIGS_JOB_FORMAT]: {
    kind: CONFIGS_JOB_FORMAT,
    source: EApplicationId.CONFIGS_FORMATTER,
    label: "Config formatting",
  },
  [CONFIGS_JOB_VERIFY]: {
    kind: CONFIGS_JOB_VERIFY,
    source: EApplicationId.CONFIGS_VERIFIER,
    label: "Config verification",
  },
  [TRANSLATIONS_JOB_BUILD]: {
    kind: TRANSLATIONS_JOB_BUILD,
    source: EApplicationId.TRANSLATIONS_BUILDER,
    label: "Translation build",
  },
  [TRANSLATIONS_JOB_CHECK_FORMAT]: {
    kind: TRANSLATIONS_JOB_CHECK_FORMAT,
    source: EApplicationId.TRANSLATIONS_FORMATTER,
    label: "Translation format check",
  },
  [TRANSLATIONS_JOB_FORMAT]: {
    kind: TRANSLATIONS_JOB_FORMAT,
    source: EApplicationId.TRANSLATIONS_FORMATTER,
    label: "Translation formatting",
  },
  [TRANSLATIONS_JOB_PARSE]: {
    kind: TRANSLATIONS_JOB_PARSE,
    source: EApplicationId.TRANSLATIONS_PARSER,
    label: "Translation import",
  },
  [TRANSLATIONS_JOB_VERIFY]: {
    kind: TRANSLATIONS_JOB_VERIFY,
    source: EApplicationId.TRANSLATIONS_VERIFIER,
    label: "Translation check",
  },
  [SPRITE_EQUIPMENT_JOB_PACK]: {
    kind: SPRITE_EQUIPMENT_JOB_PACK,
    source: EApplicationId.SPRITE_EQUIPMENT_PACKER,
    label: "Equipment sprite packing",
  },
  [GAMEDATA_JOB_VERIFY]: {
    kind: GAMEDATA_JOB_VERIFY,
    source: EApplicationId.GAMEDATA_VERIFIER,
    label: "Gamedata verification",
  },
  [LEVELS_JOB_CAPTURE_CUBE]: {
    kind: LEVELS_JOB_CAPTURE_CUBE,
    source: EApplicationId.LEVEL_VIEWER,
    label: "Cubemap capture",
  },
  [LEVELS_JOB_CAPTURE_MAP]: {
    kind: LEVELS_JOB_CAPTURE_MAP,
    source: EApplicationId.LEVEL_VIEWER,
    label: "Level map capture",
  },
  [TEXTURES_JOB_SAVE]: {
    kind: TEXTURES_JOB_SAVE,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Texture save",
  },
  [TEXTURES_JOB_BUILD]: {
    kind: TEXTURES_JOB_BUILD,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Texture build",
  },
  [TEXTURES_JOB_MAKE_BUMP]: {
    kind: TEXTURES_JOB_MAKE_BUMP,
    source: EApplicationId.TEXTURES_EXPLORER,
    label: "Bump pair generation",
  },
  [TEXTURES_JOB_COMPARE_ENCODINGS]: {
    kind: TEXTURES_JOB_COMPARE_ENCODINGS,
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
export function findJobKind(kind: JobKind): Nullable<IJobKindDescriptor> {
  return JOB_KINDS[kind] ?? null;
}
