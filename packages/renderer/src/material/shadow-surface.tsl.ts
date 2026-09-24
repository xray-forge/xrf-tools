import { Nullable } from "@xrf/types";
import { Discard, Fn, If, uniform, vec4 } from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { toSurfaceCoordinates } from "#/material/surface-texel.tsl";
import { getWhiteTexture } from "#/texture/placeholder-textures";

/** `def_aref`: where a cut-out surface without its own reference is cut, as the G-buffer cuts it. */
const DEFAULT_ALPHA_REFERENCE: number = 200 / 255;

/**
 * A surface as a shadow map sees it: its depth alone. A cut-out surface is cut where the G-buffer cuts it, so a leaf
 * card's shadow has the leaf's edge rather than the card's.
 *
 * @param surface - The cut-out surface casting, or null for any opaque one.
 * @param samplers - Where its slots are bound, null for an opaque one.
 * @returns Its shader, writing no colour.
 */
export function toShadowSurfaceShader(
  surface: Nullable<IRendererSurface>,
  samplers: Nullable<MaterialSamplers>
): ISurfaceShader {
  if (!surface || !samplers || surface.draw !== ERendererDraw.CUT_OUT) {
    return { fragmentNode: vec4(0) };
  }

  const base: TextureNode = samplers.bind(surface.textures.base, getWhiteTexture(), toSurfaceCoordinates(surface));
  const reference: Node<"float"> = uniform(surface.alphaReference ?? DEFAULT_ALPHA_REFERENCE);

  return {
    fragmentNode: Fn(() => {
      If(base.w.lessThanEqual(reference), () => {
        Discard();
      });

      return vec4(0);
    })(),
  };
}
