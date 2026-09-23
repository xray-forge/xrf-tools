import { Nullable } from "@xrf/types";
import { MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererPass, IRendererSurface, toRendererPass } from "#/contract/scene/renderer-surface";
import { toDeferredSurfaceShader } from "#/material/deferred-surface.tsl";
import { toForwardSurfaceShader } from "#/material/forward-surface.tsl";
import { MaterialSamplers } from "#/material/material-samplers";
import { applySurfaceCompositing, ISurfaceCompositing, toSurfaceCompositing } from "#/material/surface-compositing";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { ISurfaceShader } from "#/material/surface-shader";
import { toWallmarkSurfaceShader } from "#/material/wallmark-surface.tsl";
import { instancedPosition } from "#/shader/placement.tsl";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** How each pass shades the surfaces it draws. */
const SURFACE_SHADERS: Record<
  ERendererPass,
  (surface: IRendererSurface, samplers: MaterialSamplers, uniforms: RendererUniforms) => ISurfaceShader
> = {
  [ERendererPass.DEFERRED]: toDeferredSurfaceShader,
  [ERendererPass.FORWARD]: toForwardSurfaceShader,
  [ERendererPass.WALLMARK]: toWallmarkSurfaceShader,
};

/**
 * A surface as the frame draws it.
 */
export interface ISurfaceMaterial {
  material: MeshBasicNodeMaterial;
  /** Which pass draws it. */
  pass: ERendererPass;
  /** The texture keys it samples, which have to be uploaded before it draws without a stall. */
  keys: ReadonlyArray<string>;
  dispose(): void;
}

/**
 * @param surface - What the consumer put.
 * @param textures - Where its textures are bound from.
 * @param uniforms - What the frame's shaders read.
 * @returns The material, shaded by the pass its draw puts it in.
 */
export function createSurfaceMaterial(
  surface: IRendererSurface,
  textures: RendererTextures,
  uniforms: RendererUniforms
): ISurfaceMaterial {
  const pass: ERendererPass = toRendererPass(surface);
  const samplers: MaterialSamplers = new MaterialSamplers(textures);
  const shader: ISurfaceShader = SURFACE_SHADERS[pass](surface, samplers, uniforms);
  const compositing: Nullable<ISurfaceCompositing> = toSurfaceCompositing(surface);
  const material: MeshBasicNodeMaterial = new SurfaceNodeMaterial(uniforms.staticDraws);

  // Every surface stands its geometry in each place instanced attributes name, and in its own place where none do.
  material.positionNode = instancedPosition();
  material.fragmentNode = shader.fragmentNode ?? null;
  material.colorNode = shader.colorNode ?? null;
  material.alphaTestNode = shader.alphaTestNode ?? null;

  if (compositing) {
    applySurfaceCompositing(material, compositing);
  }

  return {
    dispose: () => {
      samplers.release();
      material.dispose();
    },
    keys: samplers.keys,
    material,
    pass,
  };
}
