import { Nullable } from "@xrf/types";
import { vec4 } from "three/tsl";
import { TextureNode } from "three/webgpu";

import { ISurfaceInputs } from "#/material/surface-inputs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot } from "#/material/surface-slot";
import { toSurfaceCoordinates } from "#/material/surface-texel.tsl";
import { toAlphaCut } from "#/shader/alpha-cut.tsl";

/**
 * A surface as a shadow map sees it: its depth alone. A cut-out surface is cut where the G-buffer cuts it, so a leaf
 * card's shadow has the leaf's edge rather than the card's.
 *
 * @param inputs - What a cut-out caster's material carries, unbiased, or null for every opaque one.
 * @returns Its shader, writing no colour.
 */
export function toShadowSurfaceShader(inputs: Nullable<ISurfaceInputs>): ISurfaceShader {
  if (!inputs) {
    return { fragmentNode: vec4(0) };
  }

  const base: TextureNode = inputs.sample(ESurfaceSlot.BASE, toSurfaceCoordinates(inputs));

  // Cut where the G-buffer cuts it.
  return { fragmentNode: toAlphaCut(base.w, vec4(0), inputs.alphaReference) };
}
