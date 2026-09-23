import { Maybe } from "@xrf/types";
import {
  float,
  mix,
  modelViewMatrix,
  mrt,
  normalize,
  normalView,
  positionView,
  select,
  uv,
  varying,
  vec3,
  vec4,
} from "three/tsl";
import {
  CustomBlending,
  Data3DTexture,
  DstColorFactor,
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
import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { IBaseShadingPoint, toBaseColor, toFinishedColor, toSunLight } from "#/graph/base-lighting.tsl";
import { decodeBumpGloss, decodeBumpNormal } from "#/graph/bump.tsl";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { encodeOctahedral } from "#/graph/octahedral-normal.tsl";
import { EGBufferTarget } from "#/graph/renderer-targets";
import { SettingsUniforms } from "#/graph/settings-uniforms";
import { skinnedBinormal, skinnedTangent } from "#/graph/skinned-basis.tsl";
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
 * What a surface's shading reads besides its own textures: the settings, and the lighting a forward surface applies.
 */
export interface ISurfaceShadingContext {
  settings: SettingsUniforms;
  lighting: BaseLightingUniforms;
  camera: CameraUniforms;
  lut: Data3DTexture;
}

/**
 * A surface as the frame draws it.
 */
export interface ISurfaceMaterial {
  material: MeshBasicNodeMaterial;
  /** Whether it fills the G-buffer, rather than being composited after it. */
  isDeferred: boolean;
  dispose(): void;
}

/** A surface at one texel before any light: what the G-buffer stores, and what the forward path lights. */
interface ISurfaceTexel {
  base: TextureNode;
  albedo: Node<"vec3">;
  normal: Node<"vec3">;
  gloss: Node<"float">;
  /** The lightmap: hemisphere occlusion in alpha, sun occlusion in green. */
  hemi: Node<"vec4">;
  slice: Node<"float">;
}

/**
 * @param surface - What the consumer put.
 * @param textures - Where its textures are bound from.
 * @param context - The settings and lighting its shading reads.
 * @returns The material, deferred or forward as its draw decides.
 */
export function createSurfaceMaterial(
  surface: IRendererSurface,
  textures: RendererTextures,
  context: ISurfaceShadingContext
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

  const texel: ISurfaceTexel = toSurfaceTexel(surface, bind, baseCoordinates, context.settings);
  const isDeferred: boolean = surface.draw === ERendererDraw.OPAQUE || surface.draw === ERendererDraw.CUT_OUT;
  const material: MeshBasicNodeMaterial = isDeferred
    ? createDeferredMaterial(surface, texel)
    : createForwardMaterial(surface, texel, context);

  return {
    dispose: () => {
      bound.forEach(([key, sampler]) => textures.unbind(key, sampler));
      material.dispose();
    },
    isDeferred,
    material,
  };
}

/** `sload`: the surface at a texel, with the bump pair's normal and gloss where it binds one. */
function toSurfaceTexel(
  surface: IRendererSurface,
  bind: TBind,
  baseCoordinates: Node<"vec2">,
  settings: SettingsUniforms
): ISurfaceTexel {
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

  if (surface.textures.bump && surface.textures.bumpCompanion) {
    const bump: TextureNode = bind(surface.textures.bump, getFlatBumpTexture());
    const companion: TextureNode = bind(surface.textures.bumpCompanion, getFlatBumpCompanionTexture());
    const tangentSpace: Node<"vec3"> = decodeBumpNormal(bump, companion);
    // `deffer_model_bump`: the authored basis through the model view, the decoded normal rotated along it.
    const tangent: Node<"vec3"> = varying(modelViewMatrix.mul(vec4(skinnedTangent, 0)).xyz);
    const binormal: Node<"vec3"> = varying(modelViewMatrix.mul(vec4(skinnedBinormal(), 0)).xyz);
    const bumped: Node<"vec3"> = normalize(
      normalize(tangent)
        .mul(tangentSpace.x)
        .add(normalize(binormal).mul(tangentSpace.y))
        .add(normalView.mul(tangentSpace.z))
    );

    // Mixed by the switch rather than selected: a `select` between these two came out zero in a forward material.
    normal = mix(normalView, bumped, settings.bumped);
    gloss = mix(float(DEFAULT_GLOSS), decodeBumpGloss(bump), settings.bumped);
  }

  return {
    albedo,
    base,
    gloss,
    // `get_hemi` and `get_sun`: the lightmap's alpha and green; a surface without one is lit in full.
    hemi: surface.textures.hemi ? bind(surface.textures.hemi, getWhiteTexture(), uv(1)) : vec4(1),
    normal,
    slice: float(((surface.material ?? DEFAULT_MATERIAL) + 0.5) / MATERIAL_SLICES),
  };
}

/** `deffer_base` and `deffer_base_bump`: raw albedo and gloss, the view normal, and the baked occlusions. */
function createDeferredMaterial(surface: IRendererSurface, texel: ISurfaceTexel): MeshBasicNodeMaterial {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();

  material.mrtNode = mrt({
    [EGBufferTarget.ALBEDO]: vec4(texel.albedo, texel.gloss),
    [EGBufferTarget.NORMAL]: vec4(encodeOctahedral(texel.normal), 0, 1),
    [EGBufferTarget.SURFACE]: vec4(texel.hemi.w, texel.hemi.y, texel.slice, 0),
  });

  if (surface.draw === ERendererDraw.CUT_OUT) {
    // Only a cut-out surface reads the base's alpha, so a DXT1 file's punch-through never holes an opaque one.
    material.colorNode = texel.base;
    material.alphaTestNode = float(surface.alphaReference ?? DEFAULT_ALPHA_REFERENCE);
  }

  return material;
}

/**
 * A surface composited over the tonemapped frame, blended the way its draw says.
 * Blended surfaces are lit per pixel by the deferred passes' model, bump included; added and multiplied stay unlit.
 */
function createForwardMaterial(
  surface: IRendererSurface,
  texel: ISurfaceTexel,
  { settings, lighting, camera, lut }: ISurfaceShadingContext
): MeshBasicNodeMaterial {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
  let color: Node<"vec3"> = texel.albedo;

  if (surface.draw === ERendererDraw.BLENDED && surface.isLit !== false) {
    const point: IBaseShadingPoint = { normal: texel.normal, position: positionView, slice: texel.slice };
    const light: Node<"vec4"> = toSunLight(point, lighting, lut);
    const lit: Node<"vec3"> = toFinishedColor(
      toBaseColor(texel.albedo, texel.gloss, light, texel.hemi.w, point, lighting, camera, lut),
      positionView,
      lighting
    );

    color = select(settings.lit.greaterThan(0.5), lit, texel.albedo);
  }

  material.colorNode = vec4(color, texel.base.w);
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
