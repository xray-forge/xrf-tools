import { assertExhaustive } from "@xrf/types";

import { CubeExportFormat, CubeExportRequest } from "@/core/ipc/types/xrf-app";
import { ETextureEncodingFormat, ETextureEncodingQuality } from "@/core/ipc/types/xrf-texture";

/** What a cubemap is exported as, by the `kind` its export format is told apart by. */
export type TLevelCubeExportKind = CubeExportFormat["kind"];

/** A cubemap's export, as its dialog holds it. */
export interface ILevelCubeExportSettings {
  kind: TLevelCubeExportKind;
  /** The DDS's layout. */
  format: ETextureEncodingFormat;
  /** Pixels a face takes on a side, at most the capture's. */
  size: number;
  /** Whether a DDS's faces carry their whole mip chains. */
  isMipmapped: boolean;
}

/** The face sizes a cubemap is exported at, at most the capture's: the smallest for the `#small` a sky lights by. */
export const LEVEL_CUBE_EXPORT_SIZES: ReadonlyArray<number> = [32, 64, 128, 256, 512, 1024, 2048];

/** A cubemap exported as one BC7 cube DDS at the capture's own size, mipmapped. */
export const DEFAULT_LEVEL_CUBE_EXPORT: ILevelCubeExportSettings = {
  kind: "dds",
  format: ETextureEncodingFormat.BC7,
  size: Math.max(...LEVEL_CUBE_EXPORT_SIZES),
  isMipmapped: true,
};

/**
 * @param kind - What a cubemap is exported as.
 * @returns Whether it is one file, rather than a file a face written into a folder.
 */
export function isLevelCubeExportSingle(kind: TLevelCubeExportKind): boolean {
  return kind === "dds";
}

/**
 * @param settings - The export asked for.
 * @returns What the backend writes it as.
 */
export function toCubeExportFormat(settings: ILevelCubeExportSettings): CubeExportFormat {
  switch (settings.kind) {
    case "dds":
      return {
        kind: "dds",
        format: settings.format,
        quality: ETextureEncodingQuality.SLOW,
        isMipmapped: settings.isMipmapped,
      };
    case "png":
      return { kind: "png" };
    case "tga":
      return { kind: "tga" };
    default:
      return assertExhaustive(settings.kind);
  }
}

/**
 * @param settings - The export asked for.
 * @param path - The capture's cross PNG.
 * @param target - The DDS written, or the folder the faces are written into.
 * @param captured - The capture's face size, which the export takes at most.
 * @returns What the backend is asked.
 */
export function toCubeExportRequest(
  settings: ILevelCubeExportSettings,
  path: string,
  target: string,
  captured: number
): CubeExportRequest {
  return {
    path,
    target,
    format: toCubeExportFormat(settings),
    size: Math.min(settings.size, captured),
  };
}
