import { BufferGeometry } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";

/**
 * What an object's parts draw over, and the vertex layout their materials compile against for it.
 */
export interface ISceneObjectDraw {
  drawn: BufferGeometry;
  layout: string;
}

/**
 * @param surface - A surface an object draws.
 * @returns Whether its parts can be static draws: filling the G-buffer, and as triangles rather than their edges,
 *   which an indirect draw of triangle ranges cannot draw.
 */
export function isStaticSurface(surface: ISurfaceMaterial): boolean {
  return surface.pass === ERendererPass.DEFERRED && !surface.material.wireframe;
}
