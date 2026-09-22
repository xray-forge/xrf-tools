import { toRadians } from "@xrf/math";

/**
 * How a preview is lit, which is the viewer's own answer rather than anything an X-Ray file carries.
 */
export interface IRenderLighting {
  /** Degrees above the horizon the light sits at, `90` being directly overhead. */
  sunElevation: number;
  /** Degrees around the vertical axis, `0` looking along the scene's own `+z`. */
  sunAzimuth: number;
  sunIntensity: number;
  /** Hex colour of the directional light. */
  sunColor: number;
  /** Uniform light standing in for everything the one direction does not reach. */
  ambientIntensity: number;
  ambientColor: number;
}

/**
 * Where the light comes from, in the renderer's own axes.
 *
 * @param lighting - The lighting to read.
 * @param distance - How far out to put it, which only has to clear what it lights.
 * @returns The position, ready to assign.
 */
export function toRenderSunPosition(lighting: IRenderLighting, distance: number): [number, number, number] {
  const elevation: number = toRadians(lighting.sunElevation);
  const azimuth: number = toRadians(lighting.sunAzimuth);
  const horizontal: number = Math.cos(elevation) * distance;

  return [horizontal * Math.sin(azimuth), Math.sin(elevation) * distance, horizontal * Math.cos(azimuth)];
}

/** The bounds each value is offered between, so every surface adjusting lighting offers the same range. */
export const RENDER_LIGHTING_LIMITS = {
  ambientIntensity: { max: 4, min: 0, step: 0.05 },
  sunAzimuth: { max: 180, min: -180, step: 1 },
  sunElevation: { max: 90, min: -15, step: 1 },
  sunIntensity: { max: 6, min: 0, step: 0.05 },
} as const;
