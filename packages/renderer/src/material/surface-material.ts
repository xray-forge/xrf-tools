import { Nullable } from "@xrf/types";
import { DoubleSide, MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererDraw, ERendererPass, IRendererSurface, toRendererPass } from "#/contract/scene/renderer-surface";
import { toDeferredSurfaceShader } from "#/material/deferred-surface.tsl";
import { toForwardSurfaceShader } from "#/material/forward-surface.tsl";
import { MaterialSamplers } from "#/material/material-samplers";
import { toShadowSurfaceShader } from "#/material/shadow-surface.tsl";
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
  /** Whether it draws impostors, which only the LOD cull decides are drawn. */
  isImpostor: boolean;
  /**
   * What draws it into the sun's shadow maps, its depth alone; null for a surface that casts none: anything the
   * G-buffer does not draw, and an impostor, since the shadow phase casts every clump as its trees.
   */
  shadow: Nullable<MeshBasicNodeMaterial>;
  /** The texture keys its shadow material samples: a cut-out's base, and none for the one every opaque surface shares. */
  shadowKeys: ReadonlyArray<string>;
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
  const samplers: MaterialSamplers = new MaterialSamplers(textures, uniforms.settings.textureBias);
  const shader: ISurfaceShader = SURFACE_SHADERS[pass](surface, samplers, uniforms);
  const compositing: Nullable<ISurfaceCompositing> = toSurfaceCompositing(surface);
  const material: SurfaceNodeMaterial = new SurfaceNodeMaterial(uniforms.staticDraws, uniforms.wind);

  // Every surface stands its geometry in each place instanced attributes name, and in its own place where none do.
  material.positionNode = instancedPosition();
  material.fragmentNode = shader.fragmentNode ?? null;
  material.colorNode = shader.colorNode ?? null;
  material.alphaTestNode = shader.alphaTestNode ?? null;
  material.positionViewNode = shader.positionViewNode ?? null;

  if (surface.isImpostor) {
    // Its quad turns to face the camera, from whichever side it is seen.
    material.side = DoubleSide;
  }

  if (compositing) {
    applySurfaceCompositing(material, compositing);
  }

  const isCasting: boolean = pass === ERendererPass.DEFERRED && !surface.isImpostor;
  const isCutOut: boolean = surface.draw === ERendererDraw.CUT_OUT;
  const shadow: Nullable<MeshBasicNodeMaterial> = !isCasting
    ? null
    : isCutOut
      ? createShadowMaterial(surface, samplers.unbiased(), uniforms)
      : getOpaqueShadowMaterial(uniforms);

  return {
    dispose: () => {
      samplers.release();
      material.dispose();

      // The opaque one is every opaque surface's, and goes with the renderer.
      if (isCutOut) {
        shadow?.dispose();
      }
    },
    isImpostor: Boolean(surface.isImpostor),
    keys: samplers.keys,
    material,
    pass,
    shadow,
    shadowKeys: isCasting && isCutOut ? samplers.keys : [],
  };
}

/** The shadow material every opaque surface of a renderer shares, by the uniforms it was made over. */
const OPAQUE_SHADOW_MATERIALS: WeakMap<RendererUniforms, MeshBasicNodeMaterial> = new WeakMap();

/**
 * @param uniforms - What the frame's shaders read.
 * @returns The shadow material every opaque surface shares: its depth alone, whatever it is dressed with, so every
 *   opaque caster of an arena is one batch.
 */
function getOpaqueShadowMaterial(uniforms: RendererUniforms): MeshBasicNodeMaterial {
  let material: MeshBasicNodeMaterial | undefined = OPAQUE_SHADOW_MATERIALS.get(uniforms);

  if (!material) {
    material = createShadowMaterial(null, null, uniforms);
    OPAQUE_SHADOW_MATERIALS.set(uniforms, material);
  }

  return material;
}

/**
 * @param surface - A cut-out surface the G-buffer draws, or null for the opaque surfaces' shared material.
 * @param samplers - Where its slots are bound, shared with its G-buffer material.
 * @param uniforms - What the frame's shaders read.
 * @returns Its depth-only material, standing its geometry where the G-buffer's does.
 */
function createShadowMaterial(
  surface: Nullable<IRendererSurface>,
  samplers: Nullable<MaterialSamplers>,
  uniforms: RendererUniforms
): MeshBasicNodeMaterial {
  const material: SurfaceNodeMaterial = new SurfaceNodeMaterial(uniforms.staticDraws, uniforms.wind);

  material.positionNode = instancedPosition();
  material.fragmentNode = toShadowSurfaceShader(surface, samplers).fragmentNode ?? null;
  material.colorWrite = false;
  // Both faces: a card seen from the sun's side is its back as often as its front, and a wall casts either way.
  material.side = DoubleSide;

  return material;
}
