import { assertExhaustive, Nullable } from "@xrf/types";

import { EMapExportAnnotations, EMapExportFormat, MapExportFormat, MapExportRequest } from "@/core/ipc/types/xrf-app";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { RENDER_MAP_MOST_PIXELS } from "@/core/ipc/types/xrf-renderer";
import { ETextureEncodingFormat, ETextureEncodingQuality } from "@/core/ipc/types/xrf-texture";

import { LEVEL_MAP_ANNOTATIONS_EXTENSION } from "./level-map-annotations";

/** A level map's export, as its dialog holds it. */
export interface ILevelMapExportSettings {
  kind: EMapExportFormat;
  /** The DDS's layout. */
  format: ETextureEncodingFormat;
  /** Pixels its longer side takes, at most the capture's. */
  longSide: number;
  /** The colour it is laid over, or null to keep its coverage in alpha. */
  backdrop: Nullable<[number, number, number]>;
  /** Whether a DDS carries its whole mip chain. */
  isMipmapped: boolean;
  /** How its annotations go in, over the map or alone, or null for the map alone. */
  annotations: Nullable<EMapExportAnnotations>;
}

/** The longer sides a map is exported at, at most the capture's. */
export const LEVEL_MAP_EXPORT_LONG_SIDES: ReadonlyArray<number> = [512, 1024, 2048, 4096, 8192, RENDER_MAP_MOST_PIXELS];

/**
 * A map exported at the shipped `map_zaton.dds`'s 2048 pixels across, its coverage kept and mipmapped, in BC7: sharper
 * than the shipped BC3 at the same size.
 */
export const DEFAULT_LEVEL_MAP_EXPORT: ILevelMapExportSettings = {
  kind: EMapExportFormat.DDS,
  format: ETextureEncodingFormat.BC7,
  longSide: 2048,
  backdrop: null,
  isMipmapped: true,
  annotations: null,
};

/** The colour a map is laid over when its coverage is dropped, unless another is picked: black, as the engine clears. */
export const LEVEL_MAP_BACKDROP: [number, number, number] = [0, 0, 0];

/**
 * @param kind - What a map would be exported as.
 * @param annotations - How its annotations go in.
 * @returns Whether it can be exported so: only its annotations alone are an SVG.
 */
export function isLevelMapExportKindOffered(
  kind: EMapExportFormat,
  annotations: Nullable<EMapExportAnnotations>
): boolean {
  return kind !== EMapExportFormat.SVG || annotations === EMapExportAnnotations.ALONE;
}

/**
 * @param current - A map's export as it stands.
 * @param patch - What of it changes.
 * @returns The export changed, as one the backend writes: an SVG whose annotations no longer stand alone is a PNG.
 */
export function toLevelMapExportSettings(
  current: ILevelMapExportSettings,
  patch: Partial<ILevelMapExportSettings>
): ILevelMapExportSettings {
  const next: ILevelMapExportSettings = { ...current, ...patch };

  return isLevelMapExportKindOffered(next.kind, next.annotations) ? next : { ...next, kind: EMapExportFormat.PNG };
}

/**
 * @param kind - What a map is exported as.
 * @returns The extension its file takes.
 */
export function toLevelMapExportExtension(kind: EMapExportFormat): string {
  switch (kind) {
    case EMapExportFormat.DDS:
      return EXrayExtension.DDS;
    case EMapExportFormat.PNG:
      return EXrayExtension.PNG;
    case EMapExportFormat.TGA:
      return EXrayExtension.TGA;
    case EMapExportFormat.SVG:
      return LEVEL_MAP_ANNOTATIONS_EXTENSION;
    default:
      return assertExhaustive(kind);
  }
}

/**
 * @param settings - The export asked for.
 * @returns What the backend writes it as.
 */
export function toMapExportFormat(settings: ILevelMapExportSettings): MapExportFormat {
  switch (settings.kind) {
    case EMapExportFormat.DDS:
      return {
        kind: EMapExportFormat.DDS,
        format: settings.format,
        quality: ETextureEncodingQuality.SLOW,
        isMipmapped: settings.isMipmapped,
      };
    case EMapExportFormat.PNG:
      return { kind: EMapExportFormat.PNG };
    case EMapExportFormat.TGA:
      return { kind: EMapExportFormat.TGA };
    case EMapExportFormat.SVG:
      return { kind: EMapExportFormat.SVG };
    default:
      return assertExhaustive(settings.kind);
  }
}

/**
 * @param settings - The export asked for.
 * @param path - The capture's PNG.
 * @param target - Where it is written.
 * @param captured - The capture's longer side, which the export takes at most.
 * @returns What the backend is asked.
 */
export function toMapExportRequest(
  settings: ILevelMapExportSettings,
  path: string,
  target: string,
  captured: number
): MapExportRequest {
  return {
    path,
    target,
    format: toMapExportFormat(settings),
    longSide: Math.min(settings.longSide, captured),
    backdrop: settings.backdrop,
    annotations: settings.annotations,
  };
}

/**
 * @param width - A capture's pixels across.
 * @param height - Its pixels down.
 * @param longSide - The export's longer side.
 * @param kind - What it is exported as.
 * @returns The export's pixels across and down, as the backend sizes it: a DDS rounded to whole four pixel blocks.
 */
export function toLevelMapExportSize(
  width: number,
  height: number,
  longSide: number,
  kind: EMapExportFormat
): [number, number] {
  const longest: number = Math.max(width, height, 1);
  const scale: number = Math.min(Math.max(longSide, 1), longest) / longest;

  function side(pixels: number): number {
    const scaled: number = Math.max(1, Math.round(pixels * scale));

    return kind === EMapExportFormat.DDS ? Math.max(1, Math.round(scaled / 4)) * 4 : scaled;
  }

  return [side(width), side(height)];
}
