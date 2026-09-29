import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { IWeatherMix } from "#/weather/weather-mix";

/**
 * What a frame of thunder is run over.
 */
export interface IWeatherThunderInput {
  thunder: IRendererThunder;
  /** The weather mixed now, which names the collection and its timings. */
  mix: IWeatherMix;
  /** Where the view stands, in engine space. */
  view: TRendererVector;
  /** Real seconds, as the frame loop counts them. */
  now: number;
  /** Whether the view lets bolts strike at all. */
  isEnabled: boolean;
}
