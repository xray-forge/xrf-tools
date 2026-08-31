import { EApplicationId } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

/**
 * Kinds of backend work a tool can start, spelled as the backend registers them.
 *
 * One spelling for each, because the same string addresses a run in the jobs registry, finds it again after a reload,
 * and decides which tool an outcome is attributed to. A constant per tool would let those drift apart silently.
 */
export enum EJobKind {
  ARCHIVES_EXTRACT = "archives.extract",
  ARCHIVES_PACK = "archives.pack",
  ARCHIVES_UNPACK = "archives.unpack",
  CONFIGS_CHECK_FORMAT = "configs.check-format",
  CONFIGS_FORMAT = "configs.format",
  CONFIGS_VERIFY = "configs.verify",
  EQUIPMENT_ICONS_PACK = "equipment-icons.pack",
  GAMEDATA_VERIFY = "gamedata.verify",
  TRANSLATIONS_BUILD = "translations.build",
  TRANSLATIONS_PARSE = "translations.parse",
  TRANSLATIONS_VERIFY = "translations.verify",
}

/**
 * What a kind of work is, independently of any run of it.
 */
export interface IJobKindDescriptor {
  kind: EJobKind;
  /** Tool the work belongs to, which is what its notifications are attributed to. */
  source: EApplicationId;
  /** What to call the work in front of a person, where the kind itself is not presentable. */
  label: string;
}

/**
 * Every kind of work this application knows how to attribute.
 *
 * Here rather than beside each tool, for the same reason `APPLICATION_CATALOG` is not assembled from the applications:
 * a job outlives the page that started it, so the window that finds it again is usually one where the owning tool was
 * never loaded. Identity that only existed inside a running tool would be unavailable in exactly the case that needs
 * it, which is how an adopted pack came to be announced as `archives.pack` rather than as the packer's work.
 */
export const JOB_KINDS: Record<EJobKind, IJobKindDescriptor> = {
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
  [EJobKind.EQUIPMENT_ICONS_PACK]: {
    kind: EJobKind.EQUIPMENT_ICONS_PACK,
    source: EApplicationId.EQUIPMENT_ICONS_PACKER,
    label: "Equipment sprite packing",
  },
  [EJobKind.GAMEDATA_VERIFY]: {
    kind: EJobKind.GAMEDATA_VERIFY,
    source: EApplicationId.GAMEDATA_VERIFIER,
    label: "Gamedata verification",
  },
};

/**
 * Looks up what a kind of work is.
 *
 * Takes a string rather than the enum, because a listing comes from the backend: a build running against a newer
 * backend can be shown a kind it has never heard of, and answering null is how it says so.
 *
 * @param kind - Kind as the backend spelled it.
 * @returns What that kind of work is, or null where this build does not know it.
 */
export function findJobKind(kind: string): Nullable<IJobKindDescriptor> {
  return JOB_KINDS[kind as EJobKind] ?? null;
}
