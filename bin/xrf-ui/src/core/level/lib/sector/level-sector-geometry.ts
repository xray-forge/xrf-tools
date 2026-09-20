import { BufferAttribute, BufferGeometry } from "three";

import { ISectorGeometryViews, ISectorSectionViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";

/** The second uv set a lightmapped surface samples, which three.js names `uv1`. */
export const LIGHTMAP_ATTRIBUTE: string = "uv1";

/** Hemisphere occlusion, one float per vertex, which no standard material reads until a shader asks for it. */
export const HEMI_ATTRIBUTE: string = "hemi";

/**
 * Builds the geometry of one packed mesh, binding only the attributes it carries.
 *
 * @param geometry - The mesh's attribute views.
 * @returns Geometry ready to draw.
 */
export function createGeometry(geometry: ISectorGeometryViews): BufferGeometry {
  const built: BufferGeometry = new BufferGeometry();

  built.setAttribute("position", new BufferAttribute(geometry.positions, 3));
  built.setIndex(new BufferAttribute(geometry.indices, 1));

  if (geometry.normals) {
    built.setAttribute("normal", new BufferAttribute(geometry.normals, 3));
  }

  if (geometry.tangents) {
    built.setAttribute("tangent", new BufferAttribute(geometry.tangents, 3));
  }

  if (geometry.binormals) {
    built.setAttribute("binormal", new BufferAttribute(geometry.binormals, 3));
  }

  if (geometry.uvs) {
    built.setAttribute("uv", new BufferAttribute(geometry.uvs, 2));
  }

  if (geometry.lightmapUvs) {
    built.setAttribute(LIGHTMAP_ATTRIBUTE, new BufferAttribute(geometry.lightmapUvs, 2));
  }

  if (geometry.colors) {
    built.setAttribute("color", new BufferAttribute(geometry.colors, 3));
  }

  if (geometry.hemi) {
    built.setAttribute(HEMI_ATTRIBUTE, new BufferAttribute(geometry.hemi, 1));
  }

  built.computeBoundingSphere();

  return built;
}

/**
 * Builds the single geometry a sector's own surfaces draw from, with a group per surface.
 *
 * @param views - The sector's attribute views and its draws.
 * @returns A geometry whose group `i` is drawn by material `i`, in the order the sections are given.
 */
export function createSectorGeometry(views: ISectorViews): BufferGeometry {
  const geometry: BufferGeometry = createGeometry(views.geometry);

  views.sections.forEach((section: ISectorSectionViews, index: number) => {
    geometry.addGroup(section.start, section.count, index);
  });

  return geometry;
}
