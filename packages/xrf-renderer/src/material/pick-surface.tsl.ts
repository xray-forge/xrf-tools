import { Nullable } from "@xrf/types";
import { TextureNode } from "three/webgpu";

import { ISurfaceInputs } from "#/material/surface-inputs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot } from "#/material/surface-slot";
import { toSurfaceCoordinates } from "#/material/surface-texel.tsl";
import { toCoverageAlpha } from "#/shader/alpha-coverage.tsl";
import { toAlphaCut } from "#/shader/alpha-cut.tsl";
import { toPickOutput } from "#/shader/pick-output.tsl";

/**
 * A surface as a pick sees it: what draws it, where it stands. A cut-out surface is cut where the G-buffer cuts it, so
 * a point between a card's leaves picks what is behind them.
 *
 * @param inputs - What a cut-out surface's material carries, unbiased, or null for every other.
 * @returns Its shader.
 */
export function toPickSurfaceShader(inputs: Nullable<ISurfaceInputs>): ISurfaceShader {
  if (!inputs) {
    return { fragmentNode: toPickOutput() };
  }

  const coordinates = toSurfaceCoordinates(inputs);
  const base: TextureNode = inputs.sample(ESurfaceSlot.BASE, coordinates);

  return {
    fragmentNode: toAlphaCut(toCoverageAlpha(base.w, coordinates, base), toPickOutput(), inputs.alphaReference),
  };
}
