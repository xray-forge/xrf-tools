import { TextureBumpGloss } from "@xrf/ipc/types/xrf-app-textures";
import { Nullable } from "@xrf/types";

/**
 * The level the generator's own field starts at.
 *
 * Comfortably above the backend's minimum gloss power: a matte surface is a deliberate act rather than the state somebody should arrive
 * in, so the form opens on a gloss that does something and lets it be turned down.
 */
export const DEFAULT_GLOSS_POWER: number = 0.5;

/**
 * Where a generated pair takes its gloss from, as the bump panel's fields have it.
 *
 * @param isFromHeight - Whether the gloss is the height map's own alpha, as the SDK's bump source carries it.
 * @param mask - A gloss mask's path, or null for none.
 * @param level - The one level used everywhere when there is neither.
 * @returns The gloss the generation is asked for.
 */
export function toBumpGloss(isFromHeight: boolean, mask: Nullable<string>, level: number): TextureBumpGloss {
  if (isFromHeight) {
    return { kind: "heightAlpha" };
  }

  return mask ? { kind: "mask", path: mask } : { kind: "constant", level };
}
