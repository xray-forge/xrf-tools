import { IRendererWeather } from "#/contract/weather/renderer-weather";

/**
 * A weather as it crosses to the worker: the parts the consumer handed over as other objects than the weather sent
 * before it, or the whole of it after none. A part handed over as the same object stays as the worker holds it.
 */
export type TRendererWeatherChange = Partial<IRendererWeather>;
