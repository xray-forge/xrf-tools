import { TRendererVector } from "@xrf/renderer";

/**
 * @param values - What a vector field was edited to.
 * @returns Its first three components, zero where it had fewer.
 */
export function toLevelWeatherTriple(values: ReadonlyArray<number>): TRendererVector {
  const [x = 0, y = 0, z = 0] = values;

  return [x, y, z];
}

/**
 * @param values - What a vector field was edited to.
 * @returns Its first four components, zero where it had fewer.
 */
export function toLevelWeatherQuad(values: ReadonlyArray<number>): readonly [number, number, number, number] {
  const [x = 0, y = 0, z = 0, w = 0] = values;

  return [x, y, z, w];
}
