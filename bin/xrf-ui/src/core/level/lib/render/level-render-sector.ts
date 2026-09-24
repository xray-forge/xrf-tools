import { IRendererBounds, IRendererGeometry, IRendererImpostors, IRendererObject } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { SectorProgressive, VisualBounds } from "@/core/ipc/types/xrf-visual";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import {
  ISectorGeometryViews,
  ISectorImpostorViews,
  ISectorInstanceViews,
  ISectorViews,
} from "@/core/level/lib/sector/level-sector-views";

/**
 * @param views - A sector as it arrived.
 * @returns What it bakes in place, as the renderer takes it: views over the one buffer it arrived in.
 */
export function toLevelSectorGeometry(views: ISectorViews): IRendererGeometry {
  return {
    ...toLevelGeometry(views.geometry),
    bounds: toLevelBounds(views.bounds),
    // Each bounded on its own, which is what it is culled by.
    groups: views.sections.map((section, slot: number) => ({
      bounds: toLevelBounds(section.bounds),
      count: section.count,
      slot,
      start: section.start,
    })),
  };
}

/**
 * @param instance - One mesh a sector stands in many places.
 * @returns The mesh, in its own space.
 */
export function toLevelInstanceGeometry(instance: ISectorInstanceViews): IRendererGeometry {
  const progressive: Nullable<SectorProgressive> = instance.progressive;

  // A progressive mesh packs every window's indices, so its one group is its whole detail, with the bands beside it.
  return {
    ...toLevelGeometry(instance.geometry),
    groups: progressive
      ? [{ count: progressive.bands[0].count, progressive, slot: 0, start: progressive.bands[0].start }]
      : [],
  };
}

/**
 * @param views - A sector as it arrived.
 * @returns What draws its baked geometry: one surface per section, in the order its groups name them.
 */
export function toLevelSectorObject(views: ISectorViews): IRendererObject {
  return {
    geometry: LEVEL_RENDER_KEYS.sector(views.sector),
    surfaces: views.sections.map((section) => LEVEL_RENDER_KEYS.surface(section.surface.shaderId)),
  };
}

/**
 * @param sector - The sector the mesh belongs to.
 * @param index - Its position among the sector's instanced meshes.
 * @param instance - The mesh and the places it stands.
 * @returns What draws it in every one of them.
 */
export function toLevelInstanceObject(sector: number, index: number, instance: ISectorInstanceViews): IRendererObject {
  return {
    geometry: LEVEL_RENDER_KEYS.instance(sector, index),
    // The engine stores a row-vector matrix row major and the renderer takes a column-vector one column major: the
    // same sixteen floats, so nothing is rearranged.
    instances: {
      hemi: instance.hemi,
      // A tree a clump's impostor stands in for is drawn only while the clump is near enough.
      impostors: instance.impostors
        ? { indices: instance.impostors, key: LEVEL_RENDER_KEYS.impostors(sector) }
        : undefined,
      transforms: instance.transforms,
    },
    surfaces: [LEVEL_RENDER_KEYS.surface(instance.surface.shaderId)],
  };
}

/**
 * @param impostors - A sector's impostors as they arrived.
 * @returns The set its trees and its impostor draws name.
 */
export function toLevelImpostors(impostors: ISectorImpostorViews): IRendererImpostors {
  return {
    corners: impostors.corners,
    factors: impostors.factors,
    normals: impostors.normals,
    spheres: impostors.spheres,
  };
}

/**
 * The quad an impostor draws over: its corners numbered in the first coordinate, which the impostor shader places by
 * the facets it chooses; a unit sphere its bounds, which each place scales to its impostor's.
 */
export function createLevelImpostorQuad(): IRendererGeometry {
  return {
    bounds: { center: [0, 0, 0], radius: 1 },
    groups: [],
    index: new Uint16Array([0, 1, 2, 3, 2, 1]),
    position: new Float32Array([-1, 0, 0, 0, 1, 0, 1, 0, 0, 0, -1, 0]),
    uv: new Float32Array([0, 0, 1, 0, 2, 0, 3, 0]),
  };
}

/**
 * @param sector - The sector the impostors belong to.
 * @param impostors - Its impostors.
 * @param group - Which of their runs, by surface.
 * @returns What draws that run: the quad in a place per impostor, each standing where its sphere does.
 */
export function toLevelImpostorObject(sector: number, impostors: ISectorImpostorViews, group: number): IRendererObject {
  const { start, count, surface } = impostors.groups[group];
  const transforms: Float32Array = new Float32Array(count * 16);
  const indices: Int32Array = new Int32Array(count);

  for (let index = 0; index < count; index += 1) {
    const at: number = (start + index) * 4;
    const radius: number = impostors.spheres[at + 3];

    // Scaled by the radius and moved to the centre, so the cull tests each place by its impostor's sphere.
    transforms.set([radius, 0, 0, 0, 0, radius, 0, 0, 0, 0, radius, 0], index * 16);
    transforms.set([impostors.spheres[at], impostors.spheres[at + 1], impostors.spheres[at + 2], 1], index * 16 + 12);
    indices[index] = start + index;
  }

  return {
    geometry: LEVEL_RENDER_KEYS.impostorQuad,
    instances: { impostors: { indices, key: LEVEL_RENDER_KEYS.impostors(sector) }, transforms },
    surfaces: [LEVEL_RENDER_KEYS.surface(surface.shaderId)],
  };
}

/** The engine's own vertex as it arrived, packed, beside float positions; a sector without normals carries none. */
function toLevelGeometry(geometry: ISectorGeometryViews): Omit<IRendererGeometry, "groups"> {
  return {
    index: geometry.indices,
    packed: geometry.normals
      ? {
          binormal: geometry.binormals ?? undefined,
          normal: geometry.normals,
          tangent: geometry.tangents ?? undefined,
          uv: geometry.uvs ?? undefined,
          uv1: geometry.lightmapUvs ?? undefined,
        }
      : undefined,
    position: geometry.positions,
  };
}

/** The packer's sphere, where it measured one: a non-finite radius crosses the wire as null. */
function toLevelBounds(bounds: Nullable<VisualBounds>): IRendererBounds | undefined {
  const sphere = bounds?.boundingSphere;

  if (!sphere || sphere.radius === null || sphere.radius === undefined) {
    return undefined;
  }

  return { center: [sphere.center.x ?? 0, sphere.center.y ?? 0, sphere.center.z ?? 0], radius: sphere.radius };
}
