/** The engine's three cascade widths in metres (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
export const RENDERER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

/** Cascades the sun's shadow can be cut into at most: the sun reads their texels from one `vec4`. */
export const RENDERER_MAX_SHADOW_CASCADES: number = 4;

/**
 * The sun's shadow: cascades of maps, each a square of the level seen from the sun, drawn every frame through the
 * static draws and sampled by the sun's light. The engine's are three, 20, 40 and 160 metres across, at 2048 texels
 * (`render_phase_sun.cpp`, `r2_smap_size`).
 */
export interface IRendererShadowSettings {
  isEnabled: boolean;
  /** Each cascade's width in metres, nearest first; as many cascades as widths, at most `RENDERER_MAX_SHADOW_CASCADES`. */
  cascades: ReadonlyArray<number>;
  /** Texels each cascade's map is across. */
  resolution: number;
  /** Texels the filter reaches from the one sampled, each way: zero for one comparison, one for a three by three. */
  filter: number;
  /**
   * How far a point is moved along its normal before it is compared, in texels of its cascade: keeps a lit surface
   *  from shadowing itself.
   */
  bias: number;
  /** Metres towards the sun past a cascade that its casters may stand: a tower outside the map still shades into it. */
  reach: number;
  /**
   * How far in from a cascade's edge, as a share of its width, the next cascade is mixed in: none switches maps at a
   * line, as the engine does.
   */
  blend: number;
  /**
   * Whether cascade `n` is drawn at most every `2^n` frames, the far ones sharing frames the near one does not. A map
   * holds depth in the world and is sampled with the matrix it was drawn with, so a map a frame or three old is exact
   * for everything that stands still; only something moving would cast late.
   */
  isStaggered: boolean;
}

/**
 * @param settings - The sun's shadow settings, held to the schema.
 * @returns Cascades drawn: as many as widths, and none while shadows are off.
 */
export function toShadowCascadeCount(settings: IRendererShadowSettings): number {
  return settings.isEnabled ? settings.cascades.length : 0;
}

/** Every cascade, the engine's three and the far one, and a filter a texel wide. */
export const DEFAULT_RENDERER_SHADOW_SETTINGS: IRendererShadowSettings = {
  bias: 1.5,
  blend: 0.1,
  cascades: RENDERER_SHADOW_CASCADE_WIDTHS,
  filter: 1,
  isEnabled: true,
  isStaggered: true,
  reach: 400,
  resolution: 2048,
};
