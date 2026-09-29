import { Mesh } from "three/webgpu";

import { IWeatherSurface } from "#/material/weather-surface";

/** One draw of the weather: its mesh, its surface, and the texture key its sampler is bound to. */
export interface IWeatherDraw {
  mesh: Mesh;
  surface: IWeatherSurface;
  key: string;
}
