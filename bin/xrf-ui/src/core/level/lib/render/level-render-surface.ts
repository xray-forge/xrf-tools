import { ERendererDraw, IRendererSurface, TRendererColor } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";
import { ILevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";

/** Turns of the golden angle, which spreads consecutive shader ids rather than grouping them into near hues. */
const HUE_STEP: number = 137.508;

/** The shader an impostor is drawn with (`details_lod.s`). */
export const LEVEL_IMPOSTOR_SHADER: string = "details\\lod";

/** What `details_lod.s` appends to the atlas for its `s_hemi`. */
export const LEVEL_IMPOSTOR_COMPANION_SUFFIX: string = "_nm";

/**
 * @param surface - A shader table entry.
 * @returns Whether it draws impostors.
 */
export function isLevelImpostorSurface(surface: SectorSurface): boolean {
  return surface.shaderName?.toLowerCase() === LEVEL_IMPOSTOR_SHADER;
}

/**
 * One shader table entry, as the renderer draws it.
 *
 * @param surface - What the level's table names for it.
 * @param render - What its blender compiles to.
 * @param options - What the toolbar has switched on.
 * @returns The surface.
 */
export function toLevelSurface(
  surface: SectorSurface,
  render: ILevelSurfaceRender,
  options: ILevelSurfaceOptions
): IRendererSurface {
  const base: Nullable<string> = options.isTextured ? surface.textureName : null;
  const detail = options.isTextured ? render.detail : null;

  if (isLevelImpostorSurface(surface)) {
    return {
      color: base ? undefined : toLevelSurfaceColor(surface.shaderId),
      draw: ERendererDraw.OPAQUE,
      isImpostor: true,
      textures: {
        base: base ?? undefined,
        hemi: surface.textureName ? `${surface.textureName}${LEVEL_IMPOSTOR_COMPANION_SUFFIX}` : undefined,
      },
    };
  }

  return {
    alphaReference: render.alphaReference,
    // An untextured surface takes its entry's colour, so a level with textures off is still read surface by surface.
    color: base ? undefined : toLevelSurfaceColor(surface.shaderId),
    detailScale: detail?.scale,
    draw: render.draw,
    isLit: render.isLit,
    isWallmark: render.isWallmark || undefined,
    textures: {
      base: base ?? undefined,
      detail: detail?.reference,
      // The row's third texture, which `uber_deffer` binds as `s_hemi`: the second is R1's baked colour.
      hemi: surface.hemi ?? undefined,
    },
  };
}

/**
 * A stable colour per shader table entry, so one surface is the same colour in every sector of the level.
 *
 * @param shaderId - Entry of the level's shader table.
 * @returns Raw red, green and blue, from a hue derived from the id rather than assigned in arrival order.
 */
export function toLevelSurfaceColor(shaderId: number): TRendererColor {
  const hue: number = (shaderId * HUE_STEP) % 360;

  return toRgb(hue, 0.45, 0.6);
}

/** Hue in degrees, saturation and lightness in `[0, 1]`, to raw red, green and blue. */
function toRgb(hue: number, saturation: number, lightness: number): TRendererColor {
  const reach: number = saturation * Math.min(lightness, 1 - lightness);

  function channel(offset: number): number {
    const turn: number = (offset + hue / 30) % 12;

    return lightness - reach * Math.max(-1, Math.min(turn - 3, 9 - turn, 1));
  }

  return [channel(0), channel(8), channel(4)];
}
