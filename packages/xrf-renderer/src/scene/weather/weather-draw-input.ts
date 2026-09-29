import { BufferGeometry } from "three/webgpu";

import { IWeatherSurface } from "#/material/weather-surface";
import { RendererTextures } from "#/texture/renderer-textures";

/** What a weather draw is made of. */
export interface IWeatherDrawInput {
  /** Where its texture is put. */
  textures: RendererTextures;
  geometry: BufferGeometry;
  surface: IWeatherSurface;
  /** Its texture, as the weather names it. */
  reference: string;
  /** Where it draws among the others, lowest first; none for the first. */
  order?: number;
}
