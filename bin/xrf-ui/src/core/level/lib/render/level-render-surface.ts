import { ERendererDraw, IRendererSurface, TRendererColor } from "@xrf/renderer";
import { Maybe } from "@xrf/types";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
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
 * One shader table entry, as the renderer draws it: with every texture it binds, whether the settings draw them or not.
 *
 * @param surface - What the level's table names for it.
 * @param render - What its blender compiles to.
 * @returns The surface.
 */
export function toLevelSurface(surface: SectorSurface, render: ILevelSurfaceRender): IRendererSurface {
  const base: Maybe<string> = surface.textureName ?? undefined;
  const { detail, bump } = render;
  // The entry's own colour, drawn without textures, so a level with them off is still read surface by surface.
  const color: TRendererColor = toLevelSurfaceColor(surface.shaderId);

  if (isLevelImpostorSurface(surface)) {
    return {
      color,
      draw: ERendererDraw.OPAQUE,
      isImpostor: true,
      textures: {
        base,
        hemi: base ? `${base}${LEVEL_IMPOSTOR_COMPANION_SUFFIX}` : undefined,
      },
    };
  }

  if (render.waterTextures) {
    const water = render.waterTextures;

    return {
      color,
      draw: render.draw,
      isLit: render.isLit,
      textures: {
        // The script's own base, which is the water's colour and the mix of its reflection, over the row's.
        base: water.base ?? base,
        distortion: water.distortion ?? undefined,
        foam: water.foam ?? undefined,
        normal: water.normal ?? undefined,
      },
      water: render.water,
    };
  }

  return {
    alphaReference: render.alphaReference,
    color,
    detailScale: detail?.scale,
    draw: render.draw,
    isLit: render.isLit,
    isWallmark: render.isWallmark || undefined,
    material: render.material,
    textures: {
      base,
      bump: bump?.bump,
      bumpCompanion: bump?.companion,
      detail: detail?.reference,
      detailBump: detail?.bump?.bump,
      detailBumpCompanion: detail?.bump?.companion,
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
