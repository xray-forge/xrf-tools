import { isRecord, Nullable } from "@xrf/types";

import { XrfApplicationError } from "@/core/error/lib";
import { BuildInfo } from "@/core/ipc/types/xrf-build-info";
import { toLevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { ELevelLookSource, toLevelLookChoice } from "@/core/level/lib/look/level-look-choice";
import { toWeatherTimeOfDay } from "@/core/level/lib/weather/weather-day";
import { TRenderExposureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { flattenDottedRecord } from "@/lib/object/dotted-record";

/** What a snapshot file says it is, so a JSON file of anything else is refused. */
export const LEVEL_VIEWER_SNAPSHOT_FORMAT: string = "xrf-level-view";

/** Decimal places a file keeps the camera's place to: a millimetre, a thousandth of a degree. */
export const LEVEL_VIEWER_SNAPSHOT_CAMERA_DECIMALS: number = 3;

/** A part of the view a snapshot holds, an object of its values. */
export type TLevelViewerSnapshotPart = Record<string, unknown>;

/**
 * A level view as a file holds it, plain JSON: what describes the file, then each part of the view as an object of its
 * own, every renderer setting as the view draws it under `settings`. A file is read before it is trusted, so each part
 * is any record until an import compares it with the view.
 */
export interface ILevelViewerSnapshot {
  format: string;
  /** The build that took it, where known. */
  build?: BuildInfo;
  exportedAt?: string;
  /** The level it was taken on, null with none open. */
  level?: Nullable<string>;
  camera?: TLevelViewerSnapshotPart;
  weather?: TLevelViewerSnapshotPart;
  look?: TLevelViewerSnapshotPart;
  output?: TLevelViewerSnapshotPart;
  options?: TLevelViewerSnapshotPart;
  settings?: TLevelViewerSnapshotPart;
}

/** What importing a value of a snapshot does. */
export enum ELevelViewerSnapshotStatus {
  /** The view draws it so already. */
  SAME = "same",
  /** The view takes it: as the file has it, or as this build bounds it. */
  CHANGE = "change",
  /** The view does not take it, which `reason` says why. */
  SKIP = "skip",
}

/** One value of a snapshot, compared with the view an import would apply it to. */
export interface ILevelViewerSnapshotEntry {
  /** Its path in the file, dotted: `settings.shadows.contact.steps`. */
  key: string;
  /** The part of the view it belongs to, which the import offers whole: `camera`, `settings.shadows`. */
  group: string;
  /** What the file holds. */
  value: unknown;
  /** What the view takes of it, undefined where it takes nothing. */
  applied: unknown;
  /** What the view has now, undefined where it has nothing there. */
  current: unknown;
  status: ELevelViewerSnapshotStatus;
  reason?: string;
}

/** What a snapshot is compared against besides the view: the level open, and what fills a partial look. */
export interface ILevelViewerSnapshotContext {
  level: Nullable<string>;
  exposure: TRenderExposureSettings;
}

/** The keys describing a file rather than holding a part of the view. */
const META_KEYS: ReadonlyArray<string> = ["format", "build", "exportedAt", "level"];

/** What the camera's place takes, by example. */
const CAMERA_PLACE: Readonly<TLevelViewerSnapshotPart> = { heading: 0, pitch: 0, x: 0, y: 0, z: 0 };

/** What the weather takes, by example. */
const WEATHER: Readonly<TLevelViewerSnapshotPart> = { cycle: "", isPlaying: false, time: 0 };

/**
 * @param text - The file's text.
 * @returns The snapshot it holds.
 * @throws {XrfApplicationError} Where the text is not JSON, not an object, or not a level view.
 */
export function parseLevelViewerSnapshot(text: string): ILevelViewerSnapshot {
  let parsed: unknown;

  try {
    parsed = JSON.parse(text);
  } catch (error: unknown) {
    throw new XrfApplicationError(`The file is not JSON: ${error instanceof Error ? error.message : String(error)}`);
  }

  if (!isRecord(parsed) || parsed.format !== LEVEL_VIEWER_SNAPSHOT_FORMAT) {
    throw new XrfApplicationError(`The file is not a level view: its format is not "${LEVEL_VIEWER_SNAPSHOT_FORMAT}".`);
  }

  return parsed as unknown as ILevelViewerSnapshot;
}

/**
 * @param snapshot - A snapshot.
 * @returns Its file's text.
 */
export function serializeLevelViewerSnapshot(snapshot: ILevelViewerSnapshot): string {
  return `${JSON.stringify(snapshot, null, 2)}\n`;
}

/**
 * Compares every value of a file with what the view would take of it and with what it draws now: the same, a change
 * (bounded where this build holds it to another value), or skipped, with why.
 *
 * @param file - The snapshot to import.
 * @param current - The view as it stands, as a snapshot of it holds it.
 * @param context - The level open, and what fills a partial look.
 * @returns Every value of the file, compared.
 */
export function compareLevelViewerSnapshot(
  file: ILevelViewerSnapshot,
  current: ILevelViewerSnapshot,
  context: ILevelViewerSnapshotContext
): Array<ILevelViewerSnapshotEntry> {
  const taken: Record<string, unknown> = flattenDottedRecord(takeLevelViewerSnapshot(file, current, context));
  const now: Record<string, unknown> = flattenDottedRecord(withoutMeta(current));

  return Object.entries(flattenDottedRecord(withoutMeta(file))).map(([key, value]: [string, unknown]) => {
    const parts: Array<string> = key.split(".");
    const entry: Omit<ILevelViewerSnapshotEntry, "status"> = {
      applied: taken[key],
      current: now[key],
      group: parts[0] === "settings" && parts.length > 2 ? `settings.${parts[1]}` : parts[0],
      key,
      value,
    };

    if (isSameValue(entry.applied ?? value, entry.current)) {
      return { ...entry, status: ELevelViewerSnapshotStatus.SAME };
    }

    return entry.applied === undefined
      ? { ...entry, reason: toSkipReason(parts, file, context), status: ELevelViewerSnapshotStatus.SKIP }
      : { ...entry, status: ELevelViewerSnapshotStatus.CHANGE };
  });
}

/**
 * The file as the view would hold it, through the readers the view keeps its own state with: the renderer settings it
 * can set for itself and its output as stored features are read, the lens as stored camera options, the look as a
 * stored look, and the rest by the kind of value each takes. The camera's place only on the level it was taken on.
 */
function takeLevelViewerSnapshot(
  file: ILevelViewerSnapshot,
  current: ILevelViewerSnapshot,
  { level, exposure }: ILevelViewerSnapshotContext
): Record<string, TLevelViewerSnapshotPart> {
  const { output, antialiasing, ...settings } = toLevelFeatureOptions({ ...file.settings, output: file.output });
  const camera: TLevelViewerSnapshotPart = file.camera ?? {};
  const look: TLevelViewerSnapshotPart = file.look ?? {};
  const weather: TLevelViewerSnapshotPart = pickLike(file.weather, WEATHER);

  return {
    camera: {
      ...(level !== null && file.level === level ? pickLike(camera, CAMERA_PLACE) : {}),
      ...(typeof camera.fieldOfView === "number"
        ? { fieldOfView: toLevelCameraOptions({ fieldOfView: camera.fieldOfView }).fieldOfView }
        : {}),
    },
    look: {
      ...(Object.values(ELevelLookSource).includes(look.source as ELevelLookSource) ? { source: look.source } : {}),
      ...(isRecord(look.custom)
        ? { custom: toLevelLookChoice({ custom: look.custom, source: ELevelLookSource.CUSTOM }, exposure).custom }
        : {}),
    },
    options: pickLike(file.options, current.options ?? {}),
    output,
    settings: { ...settings, ...(antialiasing ? { antialiasing } : {}) },
    weather: {
      ...weather,
      ...(typeof weather.time === "number" ? { time: toWeatherTimeOfDay(weather.time) } : {}),
    },
  };
}

/** Why the view takes none of a value: a place taken elsewhere, a setting it cannot set, or a value it does not read. */
function toSkipReason(
  parts: ReadonlyArray<string>,
  file: ILevelViewerSnapshot,
  context: ILevelViewerSnapshotContext
): string {
  if (parts[0] === "camera" && parts[1] in CAMERA_PLACE && file.level !== context.level) {
    return `taken on ${typeof file.level === "string" ? file.level : "a level the file does not name"}`;
  }

  return parts[0] === "settings" ? "not a setting the view sets for itself" : "not a value this build takes";
}

/** The values of a part that are of the kind the example holds at their keys. */
function pickLike(part: unknown, example: Readonly<TLevelViewerSnapshotPart>): TLevelViewerSnapshotPart {
  return Object.fromEntries(
    Object.entries(isRecord(part) ? part : {}).filter(
      ([key, value]: [string, unknown]) =>
        key in example && typeof value === typeof example[key] && (typeof value !== "number" || Number.isFinite(value))
    )
  );
}

function withoutMeta(snapshot: ILevelViewerSnapshot): Record<string, unknown> {
  return Object.fromEntries(Object.entries(snapshot).filter(([key]: [string, unknown]) => !META_KEYS.includes(key)));
}

/** The same JSON value: what a file holds is what it was written from. */
function isSameValue(a: unknown, b: unknown): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
