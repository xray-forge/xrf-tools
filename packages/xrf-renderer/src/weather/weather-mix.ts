import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * A cycle mixed at one time of day, `CEnvDescriptorMixer` after `lerp`, in engine space and units.
 */
export interface IWeatherMix {
  /** Seconds since midnight. */
  time: number;
  /** Where it was seen from, in engine space. */
  view: TRendererVector;
  /** How many of the level's modifiers reach the view. */
  modifiers: number;
  /** The keyframes either side, by index. */
  keyframes: readonly [number, number];
  /** How far from the first to the second. */
  weight: number;
  skyColor: TRendererColor;
  /** Radians. */
  skyRotation: number;
  cloudsColor: readonly [number, number, number, number];
  /** Radians. */
  cloudsRotation: number;
  farPlane: number;
  fogColor: TRendererColor;
  fogDensity: number;
  fogDistance: number;
  fogNear: number;
  fogFar: number;
  hemiColor: readonly [number, number, number, number];
  sunColor: TRendererColor;
  ambientColor: TRendererColor;
  /** Normalised, the way sunlight travels, in engine space. */
  sunDirection: TRendererVector;
  waterIntensity: number;
  rainDensity: number;
  rainColor: TRendererColor;
  windVelocity: number;
  /** Radians. */
  windDirection: number;
  treeAmplitude: number;
  treeSpeed: number;
  treeRotation: number;
  treeWave: TRendererVector;
}
