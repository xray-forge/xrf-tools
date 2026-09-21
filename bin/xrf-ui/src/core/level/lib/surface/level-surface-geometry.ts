import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import {
  ISectorGeometryViews,
  ISectorInstanceViews,
  ISectorSectionViews,
} from "@/core/level/lib/sector/level-sector-views";
import { Nullable } from "@/lib/types/general";

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

/** Indices looked at per draw when measuring a coordinate's range. */
const SPAN_SAMPLES: number = 2048;

/**
 * Counts what each shader table entry actually draws, over the sectors held.
 *
 * @param sectors - The sectors currently resident.
 * @returns How much each entry draws, keyed by shader id, with entries nothing resident draws left out.
 */
export function countLevelSurfaceGeometry(
  sectors: ReadonlyMap<number, ILoadedSector>
): ReadonlyMap<number, ILevelSurfaceGeometry> {
  const counted: Map<number, ILevelSurfaceGeometry> = new Map();

  function add(
    shaderId: number,
    drawables: number,
    triangles: number,
    geometry: ISectorGeometryViews,
    start: number,
    count: number
  ): void {
    const held: ILevelSurfaceGeometry = counted.get(shaderId) ?? {
      drawables: 0,
      narrowest: null,
      span: null,
      triangles: 0,
    };
    const drawn: Nullable<ILevelSurfaceSpan> = widen(null, geometry, start, count);

    held.drawables += drawables;
    held.triangles += triangles;
    held.span = merge(held.span, drawn);
    held.narrowest = narrower(held.narrowest, drawn);

    counted.set(shaderId, held);
  }

  for (const { views } of sectors.values()) {
    for (const section of views.sections as Array<ISectorSectionViews>) {
      add(
        section.surface.shaderId,
        section.drawables.length,
        section.triangleCount,
        views.geometry,
        section.start,
        section.count
      );
    }

    // A mesh stood in many places draws its triangles once per place, which is what the frame really costs. Its
    // coordinates are the one mesh's, so they are measured once however many places it stands in.
    for (const instance of views.instances as Array<ISectorInstanceViews>) {
      add(
        instance.surface.shaderId,
        instance.drawables.length,
        (instance.geometry.indexCount / 3) * instance.instanceCount,
        instance.geometry,
        0,
        instance.geometry.indexCount
      );
    }
  }

  return counted;
}

/** The range covering both, for the union over an entry's draws. */
function merge(span: Nullable<ILevelSurfaceSpan>, drawn: Nullable<ILevelSurfaceSpan>): Nullable<ILevelSurfaceSpan> {
  if (!span || !drawn) {
    return span ?? drawn;
  }

  return {
    uMax: Math.max(span.uMax, drawn.uMax),
    uMin: Math.min(span.uMin, drawn.uMin),
    vMax: Math.max(span.vMax, drawn.vMax),
    vMin: Math.min(span.vMin, drawn.vMin),
  };
}

/** Whichever of the two covers less of the texture, by area. */
function narrower(span: Nullable<ILevelSurfaceSpan>, drawn: Nullable<ILevelSurfaceSpan>): Nullable<ILevelSurfaceSpan> {
  if (!span || !drawn) {
    return span ?? drawn;
  }

  return area(drawn) < area(span) ? drawn : span;
}

function area(span: ILevelSurfaceSpan): number {
  return (span.uMax - span.uMin) * (span.vMax - span.vMin);
}

/**
 * Widens a range by what one draw's vertices carry, sampled across it.
 *
 * @param span - The range so far, or null before anything has been measured.
 * @param geometry - Views the draw reads its vertices through.
 * @param start - First index of the draw.
 * @param count - Indices it draws.
 * @returns The range including this draw, or what came in where there are no coordinates to read.
 */
function widen(
  span: Nullable<ILevelSurfaceSpan>,
  geometry: ISectorGeometryViews,
  start: number,
  count: number
): Nullable<ILevelSurfaceSpan> {
  const { uvs, indices } = geometry;

  if (!uvs || !indices || count <= 0) {
    return span;
  }

  const stride: number = Math.max(1, Math.floor(count / SPAN_SAMPLES));
  const end: number = Math.min(start + count, indices.length);
  let widened: Nullable<ILevelSurfaceSpan> = span;

  for (let at = start; at < end; at += stride) {
    const vertex: number = indices[at] * 2;
    const u: number = uvs[vertex];
    const v: number = uvs[vertex + 1];

    if (!Number.isFinite(u) || !Number.isFinite(v)) {
      continue;
    }

    widened = widened
      ? {
          uMax: Math.max(widened.uMax, u),
          uMin: Math.min(widened.uMin, u),
          vMax: Math.max(widened.vMax, v),
          vMin: Math.min(widened.vMin, v),
        }
      : { uMax: u, uMin: u, vMax: v, vMin: v };
  }

  return widened;
}

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
