import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * A cycle mixed at one time of day, `CEnvDescriptorMixer` after `lerp`, in engine space and units.
 */
export interface IWeatherMix {
  /** Seconds since midnight. */
  time: number;
  /** The keyframes either side, by index. */
  keyframes: readonly [number, number];
  /** How far from the first to the second. */
  weight: number;
  skyColor: TRendererColor;
  /** Radians. */
  skyRotation: number;
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
  treeAmplitude: number;
  treeSpeed: number;
  treeRotation: number;
  treeWave: TRendererVector;
}
