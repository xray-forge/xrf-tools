import { BufferAttribute, BufferGeometry } from "three";

import { ISectorSectionViews, ISectorViews } from "@/core/level/lib/level-sector-views";

/** The second uv set a lightmapped surface samples, which three.js names `uv1`. */
export const LIGHTMAP_ATTRIBUTE: string = "uv1";

/** Hemisphere occlusion, one float per vertex, which no standard material reads until a shader asks for it. */
export const HEMI_ATTRIBUTE: string = "hemi";

/**
 * Builds the single geometry one sector draws from, with a group per surface.
 *
 * @param views - The sector's attribute views and its draws.
 * @returns A geometry whose group `i` is drawn by material `i`, in the order the sections are given.
 */
export function createSectorGeometry(views: ISectorViews): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();

  geometry.setAttribute("position", new BufferAttribute(views.positions, 3));
  geometry.setIndex(new BufferAttribute(views.indices, 1));

  if (views.normals) {
    geometry.setAttribute("normal", new BufferAttribute(views.normals, 3));
  }

  if (views.tangents) {
    geometry.setAttribute("tangent", new BufferAttribute(views.tangents, 3));
  }

  if (views.binormals) {
    geometry.setAttribute("binormal", new BufferAttribute(views.binormals, 3));
  }

  if (views.uvs) {
    geometry.setAttribute("uv", new BufferAttribute(views.uvs, 2));
  }

  if (views.lightmapUvs) {
    geometry.setAttribute(LIGHTMAP_ATTRIBUTE, new BufferAttribute(views.lightmapUvs, 2));
  }

  if (views.colors) {
    geometry.setAttribute("color", new BufferAttribute(views.colors, 3));
  }

  if (views.hemi) {
    geometry.setAttribute(HEMI_ATTRIBUTE, new BufferAttribute(views.hemi, 1));
  }

  views.sections.forEach((section: ISectorSectionViews, index: number) => {
    geometry.addGroup(section.start, section.count, index);
  });

  geometry.computeBoundingSphere();

  return geometry;
}
