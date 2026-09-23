import { Maybe } from "@xrf/types";
import { float, mix, normalize, uniform, uv, varying } from "three/tsl";
import { Node, TextureNode, Vector3 } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceTexel } from "#/material/surface-texel";
import { decodeBumpGloss, decodeBumpNormal } from "#/shader/bump.tsl";
import { toPlacedNormalView, toPlacedViewDirection } from "#/shader/placement.tsl";
import { skinnedBinormal, skinnedTangent } from "#/shader/skinned-basis.tsl";
import { vertexHemi } from "#/shader/vertex-hemi.tsl";
import {
  getFlatBumpCompanionTexture,
  getFlatBumpTexture,
  getNeutralDetailTexture,
  getWhiteTexture,
} from "#/texture/placeholder-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** `def_gloss`: what a surface without a bump reflects (`shaders/r3/common_defines.h`). */
const DEFAULT_GLOSS: number = 2 / 255;

/** The texture descriptor's default lighting model: Blinn, at full weight. */
const DEFAULT_MATERIAL: number = 1;

/** Lighting model slices the material lookup holds. */
const MATERIAL_SLICES: number = 4;

// Every number a surface states is a uniform rather than a constant in its shader, so surfaces differing only in
// their numbers share one program and one pipeline: a level's shader table is hundreds of entries of a few kinds.

/**
 * @param surface - The surface sampled.
 * @returns Where its base and every slot sampled with it read: the first uv set, times the surface's tiling.
 */
export function toSurfaceCoordinates(surface: IRendererSurface): Node<"vec2"> {
  return uv().mul(uniform(surface.tiling ?? 1));
}

/**
 * @param color - The base's colour.
 * @param surface - The surface sampled.
 * @returns The colour times the surface's tint, where it gives one.
 */
export function toTintedColor(color: Node<"vec3">, surface: IRendererSurface): Node<"vec3"> {
  return surface.color ? color.mul(uniform(new Vector3(...surface.color))) : color;
}

/**
 * `sload`: the surface at a texel, with the bump pair's normal and gloss where it binds one.
 *
 * @param surface - The surface sampled.
 * @param samplers - Where its slots are bound.
 * @param uniforms - What the frame's shaders read: the settings switch the bump, the static draw buffers place it.
 * @returns The texel.
 */
export function toSurfaceTexel(
  surface: IRendererSurface,
  samplers: MaterialSamplers,
  uniforms: RendererUniforms
): ISurfaceTexel {
  const { settings, staticDraws } = uniforms;
  const coordinates: Node<"vec2"> = toSurfaceCoordinates(surface);
  const base: TextureNode = samplers.bind(surface.textures.base, getWhiteTexture(), coordinates);
  const surfaceNormal: Node<"vec3"> = toPlacedNormalView(staticDraws);
  let albedo: Node<"vec3"> = toTintedColor(base.xyz, surface);
  let normal: Node<"vec3"> = surfaceNormal;
  let gloss: Node<"float"> = float(DEFAULT_GLOSS);

  if (surface.textures.detail) {
    // `D.rgb = 2 * D.rgb * detail.rgb`, sampled at the base coordinates times the detail scale.
    const detail: TextureNode = samplers.bind(
      surface.textures.detail,
      getNeutralDetailTexture(),
      coordinates.mul(uniform(surface.detailScale ?? 1))
    );

    albedo = albedo.mul(detail.xyz).mul(2);
  }

  if (surface.textures.bump && surface.textures.bumpCompanion) {
    const bump: TextureNode = samplers.bind(surface.textures.bump, getFlatBumpTexture(), coordinates);
    const companion: TextureNode = samplers.bind(
      surface.textures.bumpCompanion,
      getFlatBumpCompanionTexture(),
      coordinates
    );
    const tangentSpace: Node<"vec3"> = decodeBumpNormal(bump, companion);
    // `deffer_model_bump`: the authored basis through the model view, the decoded normal rotated along it.
    const tangent: Node<"vec3"> = varying(toPlacedViewDirection(skinnedTangent, staticDraws));
    const binormal: Node<"vec3"> = varying(toPlacedViewDirection(skinnedBinormal(), staticDraws));
    const bumped: Node<"vec3"> = normalize(
      normalize(tangent)
        .mul(tangentSpace.x)
        .add(normalize(binormal).mul(tangentSpace.y))
        .add(surfaceNormal.mul(tangentSpace.z))
    );

    // Mixed by the switch rather than selected: a `select` between these two came out zero in a forward material.
    normal = mix(surfaceNormal, bumped, settings.bumped);
    gloss = mix(float(DEFAULT_GLOSS), decodeBumpGloss(bump), settings.bumped);
  }

  // `get_hemi` and `get_sun`: the lightmap's alpha and green, or the vertex's own hemisphere term where there is no
  // lightmap, sun unoccluded.
  const lightmap: Maybe<TextureNode> = surface.textures.hemi
    ? samplers.bind(surface.textures.hemi, getWhiteTexture(), uv(1))
    : undefined;

  return {
    albedo,
    alpha: base.w,
    gloss,
    hemi: lightmap ? lightmap.w : varying(vertexHemi()),
    normal,
    slice: uniform(((surface.material ?? DEFAULT_MATERIAL) + 0.5) / MATERIAL_SLICES),
    sun: lightmap ? lightmap.y : float(1),
  };
}
