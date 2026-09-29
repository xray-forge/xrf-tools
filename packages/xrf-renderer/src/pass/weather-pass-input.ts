import { RendererTargets } from "#/pass/renderer-targets";
import { IWeatherDrawnScene } from "#/pass/weather-drawn-scene";

/** What a weather pass draws, and where. */
export interface IWeatherPassInput {
  /** The name the frame report states it under. */
  name: string;
  /** The frame's targets, whose composite it draws over. */
  targets: RendererTargets;
  /** What it draws. */
  source: IWeatherDrawnScene;
}
