import { LevelMapDescription } from "@xrf/ipc/types/xrf-app-levels";
import {
  RENDER_MAP_MOST_PIXELS,
  RENDER_MAP_MOST_SUPERSAMPLING,
  RENDER_MAP_TILE,
  RENDER_MAP_TILE_MARGIN,
  RenderMapCapture,
} from "@xrf/ipc/types/xrf-renderer";
import { VisualBounds } from "@xrf/ipc/types/xrf-visual";
import { IPoint3 } from "@xrf/math";
import { assertExhaustive, Nullable } from "@xrf/types";

import { LEVEL_CAPTURE_DRAWN_MOST } from "@/tools/level-viewer/lib/capture/level-capture";

/** Where a level map's area comes from. */
export enum ELevelMapArea {
  /** `[level_map] bound_rect` of the level's `level.ltx`, where the PDA lays its map. */
  LEVEL_MAP = "level-map",
  /** The box every sector of the level covers, `CDemoRecord`'s fallback. */
  LEVEL_GEOMETRY = "level-geometry",
  /** Four numbers typed in, or taken from the current view. */
  CUSTOM = "custom",
}

/** What a level map is drawn with. */
export enum ELevelMapLook {
  /** `CDemoRecord`'s: the `map` weather and the level's static geometry alone. */
  MAP = "map",
  /** The view's own weather, time and look, less what a view from above cannot draw. */
  CURRENT_VIEW = "current-view",
}

/** A level map's area seen from above, engine space: the least and greatest `x` and `z`. */
export interface ILevelMapRect {
  minX: number;
  minZ: number;
  maxX: number;
  maxZ: number;
}

/** Everything a level map is captured with, as its dialog holds it. */
export interface ILevelMapParameters {
  area: ELevelMapArea;
  rect: ILevelMapRect;
  /** The heights the level is drawn between, engine `y`. */
  bottom: number;
  top: number;
  /** Pixels the map's longer side takes; the shorter follows the area's shape. */
  longSide: number;
  look: ELevelMapLook;
  /** Whether the spawned objects, the water and the shadows are drawn. */
  isSpawned: boolean;
  isWatered: boolean;
  isShadowed: boolean;
  /** Whether the grass is planted over the map, which the engine's draws none of: an experiment. */
  isGrassed: boolean;
  /** Times over each side a tile is drawn before it is reduced. */
  supersampling: number;
  /** Frames each tile settles from where it stands before the one kept. */
  settleFrames: number;
}

/** The longer sides a map is captured at. */
export const LEVEL_MAP_LONG_SIDES: ReadonlyArray<number> = [1024, 2048, 4096, 8192, RENDER_MAP_MOST_PIXELS];

/** The longer side a map is captured at by default, the shipped maps' own. */
export const LEVEL_MAP_DEFAULT_LONG_SIDE: number = 2048;

/** The longest side drawn twice over by default: past it the tiles cost too much to draw so. */
const SUPERSAMPLED_LONGEST: number = 4096;

/** The supersamplings offered, least first. */
export const LEVEL_MAP_SUPERSAMPLINGS: ReadonlyArray<number> = [1, 2, RENDER_MAP_MOST_SUPERSAMPLING];

/** Metres the area taken from the current view spans at least. */
export const LEVEL_MAP_VIEW_LEAST: number = 50;

/** Frames a tile settles for by default: what the temporal passes take to converge from a fresh start. */
export const LEVEL_MAP_SETTLE_FRAMES: number = 16;

/**
 * @param rect - An area.
 * @returns Whether it spans some ground.
 */
export function isLevelMapRectSpanning(rect: ILevelMapRect): boolean {
  return rect.minX < rect.maxX && rect.minZ < rect.maxZ;
}

/**
 * @param rect - A map's area.
 * @param longSide - Pixels its longer side takes.
 * @returns The map's pixels across and down: the shorter side following the area's shape, never stretched.
 */
export function toLevelMapSize(rect: ILevelMapRect, longSide: number): [number, number] {
  const width: number = rect.maxX - rect.minX;
  const height: number = rect.maxZ - rect.minZ;

  if (width >= height) {
    return [longSide, Math.max(1, Math.round((longSide * height) / width))];
  }

  return [Math.max(1, Math.round((longSide * width) / height)), longSide];
}

/**
 * @param rect - A map's area.
 * @param longSide - Pixels its longer side takes.
 * @returns Metres of ground one map pixel spans.
 */
export function toLevelMapMetresPerPixel(rect: ILevelMapRect, longSide: number): number {
  return Math.max(rect.maxX - rect.minX, rect.maxZ - rect.minZ) / longSide;
}

/**
 * @param longSide - Pixels a map's longer side takes.
 * @returns How many times over its tiles are drawn by default.
 */
export function toLevelMapSupersampling(longSide: number): number {
  return longSide > SUPERSAMPLED_LONGEST ? 1 : 2;
}

/**
 * @param parameters - A map's parameters.
 * @returns What the renderer is asked to capture.
 */
export function toRenderMapCapture(parameters: ILevelMapParameters): RenderMapCapture {
  const { rect, bottom, top, longSide, supersampling, settleFrames, isGrassed } = parameters;
  const [width, height]: [number, number] = toLevelMapSize(rect, longSide);

  return {
    minX: rect.minX,
    minZ: rect.minZ,
    maxX: rect.maxX,
    maxZ: rect.maxZ,
    bottom,
    top,
    width,
    height,
    supersampling,
    settleFrames,
    isGrassed,
  };
}

/** A level's box seen from above, engine space: its area and the heights it spans. */
export interface ILevelMapVolume {
  rect: ILevelMapRect;
  bottom: number;
  top: number;
}

/**
 * @param levelMap - Where the level's `level.ltx` lays its map, or null for none.
 * @returns Its area, null for none or one with a coordinate that is not a number.
 */
export function toLevelMapRect(levelMap: Nullable<LevelMapDescription>): Nullable<ILevelMapRect> {
  if (!levelMap) {
    return null;
  }

  const { minX, minZ, maxX, maxZ } = levelMap.area;

  if (minX === null || minZ === null || maxX === null || maxZ === null) {
    return null;
  }

  return { minX, minZ, maxX, maxZ };
}

/**
 * @param bounds - The box every sector covers, renderer space, as the level's description gives it; null for none.
 * @returns The same box in engine space, `z` the other way, or null for none.
 */
export function toLevelMapVolume(bounds: Nullable<VisualBounds>): Nullable<ILevelMapVolume> {
  if (!bounds) {
    return null;
  }

  const { min, max } = bounds.boundingBox;

  if (min.x === null || min.y === null || min.z === null || max.x === null || max.y === null || max.z === null) {
    return null;
  }

  return {
    rect: { minX: min.x, minZ: -max.z, maxX: max.x, maxZ: -min.z },
    bottom: min.y,
    top: max.y,
  };
}

/**
 * @param at - Where the camera stands, engine space.
 * @param bottom - The lowest height drawn.
 * @returns The ground about the camera that a map from above frames: a square centred under it, as wide as twice its
 *   height above the lowest ground, and no narrower than {@link LEVEL_MAP_VIEW_LEAST}.
 */
export function toLevelMapViewRect(at: IPoint3, bottom: number): ILevelMapRect {
  const half: number = Math.max(LEVEL_MAP_VIEW_LEAST / 2, at.y - bottom);

  return { minX: at.x - half, minZ: at.z - half, maxX: at.x + half, maxZ: at.z + half };
}

/**
 * @param width - A map's pixels across.
 * @param height - Its pixels down.
 * @returns How many tiles the renderer cuts it into.
 */
export function toLevelMapTileCount(width: number, height: number): number {
  return Math.ceil(width / RENDER_MAP_TILE) * Math.ceil(height / RENDER_MAP_TILE);
}

/**
 * @param parameters - A map's parameters.
 * @returns Whether they ask for something that can be captured: an area spanning ground, a bottom below the top, and
 *   tiles every GPU draws.
 */
export function isLevelMapCapturable(parameters: ILevelMapParameters): boolean {
  const { rect, bottom, top, longSide, supersampling } = parameters;

  return isLevelMapRectSpanning(rect) && bottom < top && isLevelMapSupersamplingDrawn(longSide, supersampling);
}

/**
 * @param longSide - A map's longer side.
 * @param supersampling - Times over its tiles are drawn.
 * @returns Whether every GPU draws its largest tile that many times over, margin included.
 */
export function isLevelMapSupersamplingDrawn(longSide: number, supersampling: number): boolean {
  return (Math.min(longSide, RENDER_MAP_TILE) + 2 * RENDER_MAP_TILE_MARGIN) * supersampling <= LEVEL_CAPTURE_DRAWN_MOST;
}

/**
 * @param area - Where a map's area comes from.
 * @param levelMap - Where the level's `level.ltx` lays its map, or null for none.
 * @param volume - The level's box.
 * @param current - The area the map has now, which a custom one keeps.
 * @returns The area it takes: the level's map, the box where there is none, or `current`.
 */
export function toLevelMapAreaRect(
  area: ELevelMapArea,
  levelMap: Nullable<ILevelMapRect>,
  volume: ILevelMapVolume,
  current: ILevelMapRect
): ILevelMapRect {
  switch (area) {
    case ELevelMapArea.LEVEL_MAP:
      return levelMap ?? volume.rect;
    case ELevelMapArea.LEVEL_GEOMETRY:
      return volume.rect;
    case ELevelMapArea.CUSTOM:
      return current;
    default:
      return assertExhaustive(area);
  }
}

/**
 * @param levelMap - Where the level's `level.ltx` lays its map, or null for none.
 * @param volume - The level's box.
 * @returns A map's parameters as its dialog opens: the level's map where it lays one, its box otherwise, its whole
 *   height, the shipped maps' size and the engine's look.
 */
export function toDefaultLevelMapParameters(
  levelMap: Nullable<ILevelMapRect>,
  volume: ILevelMapVolume
): ILevelMapParameters {
  const area: ELevelMapArea = levelMap ? ELevelMapArea.LEVEL_MAP : ELevelMapArea.LEVEL_GEOMETRY;

  return {
    area,
    rect: toLevelMapAreaRect(area, levelMap, volume, volume.rect),
    bottom: volume.bottom,
    top: volume.top,
    longSide: LEVEL_MAP_DEFAULT_LONG_SIDE,
    look: ELevelMapLook.MAP,
    isSpawned: false,
    isWatered: true,
    isShadowed: true,
    isGrassed: false,
    supersampling: toLevelMapSupersampling(LEVEL_MAP_DEFAULT_LONG_SIDE),
    settleFrames: LEVEL_MAP_SETTLE_FRAMES,
  };
}
