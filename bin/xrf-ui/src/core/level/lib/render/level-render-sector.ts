import { IRendererBounds, IRendererGeometry, IRendererObject } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { ISectorGeometryViews, ISectorInstanceViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";

/**
 * @param views - A sector as it arrived.
 * @returns What it bakes in place, as the renderer takes it: views over the one buffer it arrived in.
 */
export function toLevelSectorGeometry(views: ISectorViews): IRendererGeometry {
  return {
    ...toLevelGeometry(views.geometry),
    bounds: toLevelBounds(views.bounds),
    groups: views.sections.map((section, slot: number) => ({ count: section.count, slot, start: section.start })),
  };
}

/**
 * @param instance - One mesh a sector stands in many places.
 * @returns The mesh, in its own space.
 */
export function toLevelInstanceGeometry(instance: ISectorInstanceViews): IRendererGeometry {
  return { ...toLevelGeometry(instance.geometry), groups: [] };
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
    instances: { hemi: instance.hemi, transforms: instance.transforms },
    surfaces: [LEVEL_RENDER_KEYS.surface(instance.surface.shaderId)],
  };
}

function toLevelGeometry(geometry: ISectorGeometryViews): Omit<IRendererGeometry, "groups"> {
  return {
    hemi: geometry.hemi ?? undefined,
    index: geometry.indices,
    normal: geometry.normals ?? undefined,
    position: geometry.positions,
    uv: geometry.uvs ?? undefined,
    uv1: geometry.lightmapUvs ?? undefined,
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
