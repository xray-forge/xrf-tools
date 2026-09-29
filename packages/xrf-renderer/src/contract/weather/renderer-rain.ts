import { Nullable } from "@xrf/types";

import { IRendererRainDrop } from "#/contract/weather/renderer-rain-drop";

/**
 * What a weather's rain is drawn with, as `dxRainRender` loads it.
 */
export interface IRendererRain {
  /** The streaks' texture reference, a key of the weather's textures. */
  streak: string;
  /** The splash a drop leaves, or null for none. */
  drop: Nullable<IRendererRainDrop>;
}
