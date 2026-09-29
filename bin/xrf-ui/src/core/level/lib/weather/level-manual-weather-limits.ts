/** The bounds each number of a keyframe set by hand is offered between on a slider, in its own units. */
export const LEVEL_MANUAL_WEATHER_LIMITS = {
  cloudsRotation: { max: 360, min: 0, step: 1 },
  farPlane: { max: 3000, min: 50, step: 10 },
  fogDensity: { max: 1, min: 0, step: 0.01 },
  fogDistance: { max: 3000, min: 0, step: 10 },
  rainDensity: { max: 1, min: 0, step: 0.01 },
  skyRotation: { max: 360, min: 0, step: 1 },
  sunAltitude: { max: 180, min: -180, step: 1 },
  sunLongitude: { max: 90, min: -90, step: 1 },
  treesAmplitude: { max: 0.05, min: 0, step: 0.001 },
  treesRotation: { max: 45, min: 0, step: 1 },
  treesSpeed: { max: 5, min: 0, step: 0.05 },
  waterIntensity: { max: 1, min: 0, step: 0.01 },
  windDirection: { max: 360, min: 0, step: 1 },
  windVelocity: { max: 50, min: 0, step: 0.5 },
} as const;
