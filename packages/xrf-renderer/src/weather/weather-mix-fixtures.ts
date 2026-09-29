/// <reference types="node" />

import { readFileSync } from "node:fs";
import { join } from "node:path";

/** Written by `xrf-environment`'s ignored `writes_the_renderer_golden_vectors`, from the engine's own mixer port. */
const WEATHER_MIX_GOLDEN_PATH: string = join(
  __dirname,
  "../../../../crates/xrf-environment/resources/weather-mix.golden.json"
);

/**
 * @returns The golden vectors: every cycle `xrf-environment` mixed through a day, with its keyframes.
 */
export function readWeatherMixGolden<T>(): T {
  return JSON.parse(readFileSync(WEATHER_MIX_GOLDEN_PATH, "utf8")) as T;
}
