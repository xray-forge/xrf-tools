import { Nullable } from "@xrf/types";

import { EPickKind } from "#/shader/pick-kind";

/**
 * What a pick's texel says was drawn there: which draw, which place of it, and how far from the eye.
 */
export interface IPickTexel {
  kind: EPickKind.STATIC | EPickKind.PLAIN;
  /** A static draw's slot, or a plain part's mesh id. */
  draw: number;
  /** A static draw's place, or which of a plain part's drawn instances it is. */
  place: number;
  /** Metres from the eye, along the ray through the point picked. */
  distance: number;
}

/**
 * @param pixel - A pick's texel as read back, four floats.
 * @returns What it names, or null where nothing was drawn.
 */
export function toPickTexel(pixel: ArrayLike<number>): Nullable<IPickTexel> {
  const kind: number = Math.round(pixel[0]);

  if (kind !== EPickKind.STATIC && kind !== EPickKind.PLAIN) {
    return null;
  }

  return { distance: pixel[3], draw: Math.round(pixel[1]), kind, place: Math.round(pixel[2]) };
}
