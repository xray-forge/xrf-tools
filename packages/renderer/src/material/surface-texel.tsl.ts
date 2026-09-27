import { Maybe } from "@xrf/types";
import { float, mix, normalize, varying } from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { ISurfaceInputs } from "#/material/surface-inputs";
import { ESurfaceSlot } from "#/material/surface-slot";
import { ISurfaceTexel } from "#/material/surface-texel";
import { ISurfaceVariant } from "#/material/surface-variant";
import { decodeBumpGloss, decodeBumpNormal } from "#/shader/bump.tsl";
import { toVertexAttribute } from "#/shader/cluster-vertex.tsl";
import {
  toBaseCoordinate,
  toLightmapCoordinate,
  toSurfaceBinormal,
  toSurfaceTangent,
} from "#/shader/packed-vertex.tsl";
import { toPlacedNormalView, toPlacedViewDirection } from "#/shader/placement.tsl";
import { skinnedBinormal, skinnedTangent } from "#/shader/skinned-basis.tsl";
import { toVertexHemi } from "#/shader/vertex-hemi.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** `def_gloss`: what a surface without a bump reflects (`shaders/r3/common_defines.h`). */
export const DEFAULT_GLOSS: number = 2 / 255;

/** The texture descriptor's default lighting model: Blinn, at full weight. */
export const DEFAULT_MATERIAL: number = 1;

/** Lighting model slices the material lookup holds. */
export const MATERIAL_SLICES: number = 4;

// A surface's shader is its variant's: every texture and number it states is read from its material per object, so a
// level's shader table, hundreds of entries of a few kinds, builds a few node graphs rather than hundreds.

/**
 * @param inputs - What the material drawing carries.
 * @returns Where its base and every slot sampled with it read: the first uv set, times the surface's tiling.
 */
export function toSurfaceCoordinates(inputs: ISurfaceInputs): Node<"vec2"> {
  return toBaseCoordinate(toVertexAttribute<"vec2">("uv", "vec2")).mul(inputs.tiling);
}

/**
 * @param color - The base's colour.
 * @param variant - The surface's variant.
 * @param inputs - What the material drawing carries.
 * @returns The colour times the surface's tint, where it gives one.
 */
export function toTintedColor(color: Node<"vec3">, variant: ISurfaceVariant, inputs: ISurfaceInputs): Node<"vec3"> {
  return variant.isTinted ? color.mul(inputs.color) : color;
}

/**
 * `sload`: the surface at a texel, with the bump pair's normal and gloss where it binds one.
 *
 * @param variant - The surface's variant.
 * @param inputs - What the material drawing carries.
 * @param uniforms - What the frame's shaders read: the settings switch the bump, the static draw buffers place it.
 * @returns The texel.
 */
export function toSurfaceTexel(
  variant: ISurfaceVariant,
  inputs: ISurfaceInputs,
  uniforms: RendererUniforms
): ISurfaceTexel {
  const { settings, staticDraws } = uniforms;
  const coordinates: Node<"vec2"> = toSurfaceCoordinates(inputs);
  const base: TextureNode = inputs.sample(ESurfaceSlot.BASE, coordinates);
  const surfaceNormal: Node<"vec3"> = toPlacedNormalView(staticDraws);
  let albedo: Node<"vec3"> = toTintedColor(base.xyz, variant, inputs);
  let normal: Node<"vec3"> = surfaceNormal;
  let gloss: Node<"float"> = float(DEFAULT_GLOSS);

  if (variant.hasDetail) {
    // `D.rgb = 2 * D.rgb * detail.rgb`, sampled at the base coordinates times the detail scale.
    const detail: TextureNode = inputs.sample(ESurfaceSlot.DETAIL, coordinates.mul(inputs.detailScale));

    albedo = albedo.mul(detail.xyz).mul(2);
  }

  if (variant.hasBump) {
    const bump: TextureNode = inputs.sample(ESurfaceSlot.BUMP, coordinates);
    const companion: TextureNode = inputs.sample(ESurfaceSlot.BUMP_COMPANION, coordinates);
    const tangentSpace: Node<"vec3"> = decodeBumpNormal(bump, companion);
    // `deffer_model_bump`: the authored basis through the model view, the decoded normal rotated along it.
    const tangent: Node<"vec3"> = varying(toPlacedViewDirection(toSurfaceTangent(skinnedTangent), staticDraws));
    const binormal: Node<"vec3"> = varying(toPlacedViewDirection(toSurfaceBinormal(skinnedBinormal()), staticDraws));
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
  const lightmap: Maybe<TextureNode> = variant.hasHemi
    ? inputs.sample(ESurfaceSlot.HEMI, toLightmapCoordinate(toVertexAttribute<"vec2">("uv1", "vec2")))
    : undefined;

  return {
    albedo,
    alpha: base.w,
    gloss,
    hemi: lightmap ? lightmap.w : varying(toVertexHemi(staticDraws)),
    normal,
    slice: inputs.slice,
    sun: lightmap ? lightmap.y : float(1),
  };
}
