import { Nullable } from "@xrf/types";

import { IRendererAnomalyWater } from "#/contract/scene/renderer-anomaly-water";
import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { ERendererPass, toRendererPass } from "#/contract/scene/renderer-pass";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ESurfaceSlot } from "#/material/surface-slot";

/**
 * What decides the shape of a surface's shader, and nothing else: surfaces of one variant share one node graph, so three
 * builds it once, and differ only in the textures and numbers their materials carry.
 */
export interface ISurfaceVariant {
  pass: ERendererPass;
  draw: ERendererDraw;
  isImpostor: boolean;
  /** Whether a blended surface is lit. */
  isLit: boolean;
  /** Whether it states a flat colour. */
  hasColor: boolean;
  /** Whether it binds a base, which its flat colour stands in for where it binds none. */
  hasBase: boolean;
  /** Whether a forward surface is alpha tested. */
  isAlphaTested: boolean;
  hasDetail: boolean;
  /** Whether it binds both halves of a bump pair. */
  hasBump: boolean;
  /** Whether it binds a lightmap. */
  hasHemi: boolean;
  /** Whether water is drawn by `water_soft`, blended over the depth behind it. */
  isSoftWater: boolean;
  /** Anomaly's water model, or null for OpenXRay's or for a surface that is not water. */
  anomalyWater: Nullable<IRendererAnomalyWater>;
}

/**
 * @param surface - What the consumer put.
 * @returns Its variant.
 */
export function toSurfaceVariant(surface: IRendererSurface): ISurfaceVariant {
  const { textures } = surface;

  return {
    draw: surface.draw,
    hasBase: Boolean(textures.base),
    hasBump: Boolean(textures.bump && textures.bumpCompanion),
    hasColor: Boolean(surface.color),
    hasDetail: Boolean(textures.detail),
    hasHemi: Boolean(textures.hemi),
    isAlphaTested: surface.alphaReference !== undefined,
    isImpostor: Boolean(surface.isImpostor),
    isSoftWater: surface.draw === ERendererDraw.WATER && Boolean(surface.water?.isSoft),
    anomalyWater: surface.draw === ERendererDraw.WATER ? (surface.water?.anomaly ?? null) : null,
    isLit: surface.isLit !== false,
    pass: toRendererPass(surface),
  };
}

/**
 * @param variant - A variant.
 * @returns What identifies it, equal for equal variants.
 */
export function toSurfaceVariantKey(variant: ISurfaceVariant): string {
  return [
    variant.pass,
    variant.draw,
    variant.isImpostor,
    variant.isLit,
    variant.hasColor,
    variant.isAlphaTested,
    variant.hasBase,
    variant.hasDetail,
    variant.hasBump,
    variant.hasHemi,
    variant.isSoftWater,
    variant.anomalyWater
      ? ["isReflecting", "isSpecular", "isTransparent", "isFoamed"]
          .map((it: string) => Number(variant.anomalyWater?.[it as keyof IRendererAnomalyWater]))
          .join("")
      : "-",
  ].join(":");
}

/**
 * @param variant - A variant.
 * @returns The slots its shader samples, which are what its materials bind and wait for.
 */
export function toSampledSlots(variant: ISurfaceVariant): Array<ESurfaceSlot> {
  if (variant.pass === ERendererPass.WALLMARK) {
    return [ESurfaceSlot.BASE];
  }

  if (variant.pass === ERendererPass.WATER) {
    return [ESurfaceSlot.BASE, ESurfaceSlot.NORMAL, ESurfaceSlot.FOAM, ESurfaceSlot.DISTORTION];
  }

  const slots: Array<ESurfaceSlot> = [ESurfaceSlot.BASE];

  if (variant.hasDetail && !variant.isImpostor) {
    slots.push(ESurfaceSlot.DETAIL);
  }

  if (variant.hasBump && !variant.isImpostor) {
    slots.push(ESurfaceSlot.BUMP, ESurfaceSlot.BUMP_COMPANION);
  }

  if (variant.hasHemi) {
    slots.push(ESurfaceSlot.HEMI);
  }

  return slots;
}
