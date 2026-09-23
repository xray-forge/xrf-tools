import { Nullable } from "@xrf/types";

import {
  ISectorGeometryViews,
  ISectorInstanceViews,
  ISectorSectionViews,
  ISectorViews,
} from "@/core/level/lib/sector/level-sector-views";

import { ILevelSurfaceGeometry, ILevelSurfaceSpan } from "./level-surface-geometry";

/** Indices looked at per draw when measuring a coordinate's range. */
const SPAN_SAMPLES: number = 2048;

/**
 * Counts what each shader table entry draws in one sector, read as it arrives: its bytes go to the renderer after.
 *
 * @param views - The sector.
 * @returns How much each entry draws in it, keyed by shader id.
 */
export function countSectorSurfaceGeometry(views: ISectorViews): Map<number, ILevelSurfaceGeometry> {
  const counted: Map<number, ILevelSurfaceGeometry> = new Map();

  for (const section of views.sections as Array<ISectorSectionViews>) {
    add(
      counted,
      section.surface.shaderId,
      section.drawables.length,
      section.triangleCount,
      widen(null, views.geometry, section.start, section.count)
    );
  }

  // A mesh stood in many places draws its triangles once per place, which is what the frame really costs. Its
  // coordinates are the one mesh's, so they are measured once however many places it stands in.
  for (const instance of views.instances as Array<ISectorInstanceViews>) {
    add(
      counted,
      instance.surface.shaderId,
      instance.drawables.length,
      (instance.geometry.indexCount / 3) * instance.instanceCount,
      widen(null, instance.geometry, 0, instance.geometry.indexCount)
    );
  }

  return counted;
}

/**
 * Totals what each entry draws across sectors.
 *
 * @param sectors - Each resident sector's own count.
 * @returns How much each entry draws across them, keyed by shader id, with entries nothing draws left out.
 */
export function mergeLevelSurfaceGeometry(
  sectors: Iterable<ReadonlyMap<number, ILevelSurfaceGeometry>>
): ReadonlyMap<number, ILevelSurfaceGeometry> {
  const counted: Map<number, ILevelSurfaceGeometry> = new Map();

  for (const sector of sectors) {
    for (const [shaderId, geometry] of sector) {
      const held: Nullable<ILevelSurfaceGeometry> = counted.get(shaderId) ?? null;

      counted.set(
        shaderId,
        held
          ? {
              drawables: held.drawables + geometry.drawables,
              narrowest: narrower(held.narrowest, geometry.narrowest),
              span: merge(held.span, geometry.span),
              triangles: held.triangles + geometry.triangles,
            }
          : geometry
      );
    }
  }

  return counted;
}

function add(
  counted: Map<number, ILevelSurfaceGeometry>,
  shaderId: number,
  drawables: number,
  triangles: number,
  drawn: Nullable<ILevelSurfaceSpan>
): void {
  const held: ILevelSurfaceGeometry = counted.get(shaderId) ?? {
    drawables: 0,
    narrowest: null,
    span: null,
    triangles: 0,
  };

  held.drawables += drawables;
  held.triangles += triangles;
  held.span = merge(held.span, drawn);
  held.narrowest = narrower(held.narrowest, drawn);

  counted.set(shaderId, held);
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
