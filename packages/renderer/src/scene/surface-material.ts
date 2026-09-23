import { Maybe } from "@xrf/types";
import {
  attribute,
  float,
  modelViewMatrix,
  mrt,
  normalize,
  normalView,
  select,
  uv,
  varying,
  vec3,
  vec4,
} from "three/tsl";
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
import { decodeBumpGloss, decodeBumpNormal } from "#/graph/bump.tsl";
import { encodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { EGBufferTarget } from "#/graph/renderer-targets";
import { SettingsUniforms } from "#/graph/settings-uniforms";
import {
  getFlatBumpCompanionTexture,
  getFlatBumpTexture,
  getNeutralDetailTexture,
  getWhiteTexture,
} from "#/scene/placeholder-textures";
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
 * @param settings - The settings its shading switches on.
 * @returns The material, deferred or forward as its draw decides.
 */
export function createSurfaceMaterial(
  surface: IRendererSurface,
  textures: RendererTextures,
  settings: SettingsUniforms
): ISurfaceMaterial {
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
    ? createDeferredMaterial(surface, bind, baseCoordinates, settings)
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

/** `deffer_base` and `deffer_base_bump`: raw albedo and gloss, the view normal, and the baked occlusions. */
function createDeferredMaterial(
  surface: IRendererSurface,
  bind: TBind,
  baseCoordinates: Node<"vec2">,
  settings: SettingsUniforms
): MeshBasicNodeMaterial {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
  const base: TextureNode = bind(surface.textures.base);
  let albedo: Node<"vec3"> = toTinted(base.xyz, surface);
  let normal: Node<"vec3"> = normalView;
  let gloss: Node<"float"> = float(DEFAULT_GLOSS);

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

  if (surface.textures.bump && surface.textures.bumpCompanion) {
    const bump: TextureNode = bind(surface.textures.bump, getFlatBumpTexture());
    const companion: TextureNode = bind(surface.textures.bumpCompanion, getFlatBumpCompanionTexture());
    const tangentSpace: Node<"vec3"> = decodeBumpNormal(bump, companion);
    const isBumped: Node<"bool"> = settings.bumped.greaterThan(0.5);
    // `deffer_model_bump`: the authored basis through the model view, the decoded normal rotated along it.
    const tangent: Node<"vec3"> = varying(modelViewMatrix.mul(vec4(attribute<"vec3">("tangent", "vec3"), 0)).xyz);
    const binormal: Node<"vec3"> = varying(modelViewMatrix.mul(vec4(attribute<"vec3">("binormal", "vec3"), 0)).xyz);
    const bumped: Node<"vec3"> = normalize(
      normalize(tangent)
        .mul(tangentSpace.x)
        .add(normalize(binormal).mul(tangentSpace.y))
        .add(normalView.mul(tangentSpace.z))
    );

    normal = select(isBumped, bumped, normalView);
    gloss = select(isBumped, decodeBumpGloss(bump), float(DEFAULT_GLOSS));
  }

  material.mrtNode = mrt({
    [EGBufferTarget.ALBEDO]: vec4(albedo, gloss),
    [EGBufferTarget.NORMAL]: vec4(encodeOctahedral(normal), 0, 1),
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

  const base: TextureNode = bind(surface.textures.base);

  // todo: lit forward surfaces; Base draws them unlit, as their texture.
  material.colorNode = vec4(toTinted(base.xyz, surface), base.w);
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

/** The base times the surface's colour, where it gives one. */
function toTinted(color: Node<"vec3">, surface: IRendererSurface): Node<"vec3"> {
  return surface.color ? color.mul(vec3(...surface.color)) : color;
}
