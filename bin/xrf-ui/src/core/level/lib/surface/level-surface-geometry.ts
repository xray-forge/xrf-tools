import { Nullable } from "@xrf/types";

/** The range a coordinate covers over one entry's geometry, or null where its geometry carries none. */
export interface ILevelSurfaceSpan {
  uMin: number;
  uMax: number;
  vMin: number;
  vMax: number;
}

/** How much geometry one shader table entry draws, across the sectors currently held. */
export interface ILevelSurfaceGeometry {
  /** Drawables of the level's visuals run that name the entry. */
  drawables: number;
  triangles: number;
  /** What its base coordinate covers over every draw together. */
  span: Nullable<ILevelSurfaceSpan>;
  /** The narrowest range any single draw covers. */
  narrowest: Nullable<ILevelSurfaceSpan>;
}

/** What an entry no resident sector draws comes to, so a reader is told rather than left to guess. */
export const NO_LEVEL_SURFACE_GEOMETRY: ILevelSurfaceGeometry = {
  drawables: 0,
  narrowest: null,
  span: null,
  triangles: 0,
};

/**
 * What one entry draws, in a line.
 *
 * @param geometry - What the entry draws across the resident sectors.
 * @returns A phrase naming the drawables, the triangles, and how many triangles each drawable averages.
 */
export function describeLevelSurfaceGeometry(geometry: ILevelSurfaceGeometry): string {
  if (!geometry.drawables) {
    return "nothing resident draws it";
  }

  const triangles: number = Math.round(geometry.triangles);
  const each: string = (triangles / geometry.drawables).toFixed(1);

  return `${geometry.drawables} drawables · ${triangles} triangles · ${each} each`;
}

/**
 * What one entry's base coordinate covers, in a line.
 *
 * @param span - The range, or null where its geometry carries no coordinates.
 * @returns The range in both axes, or why there is none.
 */
export function describeLevelSurfaceSpan(span: Nullable<ILevelSurfaceSpan>): string {
  if (!span) {
    return "its geometry carries no base coordinate";
  }

  return `u ${span.uMin.toFixed(2)}..${span.uMax.toFixed(2)} · ` + `v ${span.vMin.toFixed(2)}..${span.vMax.toFixed(2)}`;
}
