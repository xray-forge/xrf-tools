import { Maybe } from "@xrf/types";
import { float, mrt, normalView, uv, vec4 } from "three/tsl";
import {
  CustomBlending,
  DstColorFactor,
  Material,
  MeshBasicNodeMaterial,
  Node,
  OneFactor,
  OneMinusSrcAlphaFactor,
  SrcAlphaFactor,
  SrcColorFactor,
  Texture,
  TextureNode,
  ZeroFactor,
} from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { encodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { EGBufferTarget } from "#/graph/renderer-targets";
import { getNeutralDetailTexture, getWhiteTexture } from "#/scene/placeholder-textures";
import { RendererTextures } from "#/scene/renderer-textures";

/** `def_gloss`: what a surface without a bump reflects (`shaders/r3/common_defines.h`). */
const DEFAULT_GLOSS: number = 2 / 255;

/** `def_aref`: where a cut-out surface without its own reference is cut. */
const DEFAULT_ALPHA_REFERENCE: number = 200 / 255;

/** The texture descriptor's default lighting model: Blinn, at full weight. */
const DEFAULT_MATERIAL: number = 1;

/** Lighting model slices the material lookup holds. */
const MATERIAL_SLICES: number = 4;

/** Binds one texture slot and remembers it, so the material's disposal can let it go. */
type TBind = (key: Maybe<string>, placeholder?: Texture, coordinates?: Node<"vec2">) => TextureNode;

/**
 * A surface as the frame draws it.
 */
export interface ISurfaceMaterial {
  material: Material;
  /** Whether it fills the G-buffer, rather than being composited after it. */
  isDeferred: boolean;
  dispose(): void;
}

/**
 * @param surface - What the consumer put.
 * @param textures - Where its textures are bound from.
 * @returns The material, deferred or forward as its draw decides.
 */
export function createSurfaceMaterial(surface: IRendererSurface, textures: RendererTextures): ISurfaceMaterial {
  const bound: Array<[Maybe<string>, TextureNode]> = [];
  const baseCoordinates: Node<"vec2"> = uv().mul(surface.tiling ?? 1);

  function bind(
    key: Maybe<string>,
    placeholder: Texture = getWhiteTexture(),
    coordinates = baseCoordinates
  ): TextureNode {
    const sampler: TextureNode = textures.bind(key, placeholder, coordinates);

    bound.push([key, sampler]);

    return sampler;
  }

  const isDeferred: boolean = surface.draw === ERendererDraw.OPAQUE || surface.draw === ERendererDraw.CUT_OUT;
  const material: MeshBasicNodeMaterial = isDeferred
    ? createDeferredMaterial(surface, bind, baseCoordinates)
    : createForwardMaterial(surface, bind);

  return {
    dispose: () => {
      bound.forEach(([key, sampler]) => textures.unbind(key, sampler));
      material.dispose();
    },
    isDeferred,
    material,
  };
}

/** `deffer_base`: raw albedo and gloss, the view normal, and the baked occlusions. */
function createDeferredMaterial(
  surface: IRendererSurface,
  bind: TBind,
  baseCoordinates: Node<"vec2">
): MeshBasicNodeMaterial {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
  const base: TextureNode = bind(surface.textures.base);
  let albedo: Node<"vec3"> = base.xyz;

  if (surface.textures.detail) {
    // `D.rgb = 2 * D.rgb * detail.rgb`, sampled at the base coordinates times the detail scale.
    const detail: TextureNode = bind(
      surface.textures.detail,
      getNeutralDetailTexture(),
      baseCoordinates.mul(surface.detailScale ?? 1)
    );

    albedo = albedo.mul(detail.xyz).mul(2);
  }

  // `get_hemi` and `get_sun`: the lightmap's alpha and green; a surface without one is lit in full.
  const hemi: Node<"vec4"> = surface.textures.hemi ? bind(surface.textures.hemi, getWhiteTexture(), uv(1)) : vec4(1);
  const slice: number = ((surface.material ?? DEFAULT_MATERIAL) + 0.5) / MATERIAL_SLICES;

  // todo: gloss and the perturbed normal from the bump pair, with its decoding in iteration 2c.
  material.mrtNode = mrt({
    [EGBufferTarget.ALBEDO]: vec4(albedo, DEFAULT_GLOSS),
    [EGBufferTarget.NORMAL]: vec4(encodeOctahedral(normalView), 0, 1),
    [EGBufferTarget.SURFACE]: vec4(hemi.w, hemi.y, slice, 0),
  });

  if (surface.draw === ERendererDraw.CUT_OUT) {
    // Only a cut-out surface reads the base's alpha, so a DXT1 file's punch-through never holes an opaque one.
    material.colorNode = base;
    material.alphaTestNode = float(surface.alphaReference ?? DEFAULT_ALPHA_REFERENCE);
  }

  return material;
}

/** A surface composited over the tonemapped frame, blended the way its draw says. */
function createForwardMaterial(surface: IRendererSurface, bind: TBind): MeshBasicNodeMaterial {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();

  // todo: lit forward surfaces; Base draws them unlit, as their texture.
  material.colorNode = bind(surface.textures.base);
  material.depthWrite = false;
  material.transparent = true;
  material.blending = CustomBlending;
  // Alpha keeps what is under it, except where a blended surface covers it, so the canvas composites correctly.
  material.blendSrcAlpha = ZeroFactor;
  material.blendDstAlpha = OneFactor;

  if (surface.alphaReference !== undefined) {
    material.alphaTestNode = float(surface.alphaReference);
  }

  switch (surface.draw) {
    case ERendererDraw.BLENDED:
      material.blendSrc = SrcAlphaFactor;
      material.blendDst = OneMinusSrcAlphaFactor;
      material.blendSrcAlpha = OneFactor;
      material.blendDstAlpha = OneMinusSrcAlphaFactor;
      break;

    case ERendererDraw.ADDED:
      material.blendSrc = OneFactor;
      material.blendDst = OneFactor;
      break;

    case ERendererDraw.MULTIPLIED:
      material.blendSrc = DstColorFactor;
      material.blendDst = ZeroFactor;
      break;

    case ERendererDraw.MULTIPLIED_2X:
      material.blendSrc = DstColorFactor;
      material.blendDst = SrcColorFactor;
      break;

    case ERendererDraw.INVISIBLE:
      material.colorWrite = false;
      break;
  }

  return material;
}
