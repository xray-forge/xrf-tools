import { Nullable } from "@xrf/types";

import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * One keyframe as the engine holds it once loaded (`CEnvDescriptor`), in engine space and units.
 */
export interface IRendererWeatherKeyframe {
  /** Seconds since midnight. */
  time: number;
  /** The `sky_texture` reference, a key of the weather's textures. */
  skyTexture: string;
  /** Its `#small` irradiance cube, a key of the weather's textures. */
  skyTextureEnv: string;
  skyColor: TRendererColor;
  /** Radians. */
  skyRotation: number;
  /** The `clouds_texture` reference, a key of the weather's textures; empty for none. */
  cloudsTexture: string;
  /** The colour scaled by its multiplier, the cover in alpha. */
  cloudsColor: readonly [number, number, number, number];
  /** Radians. */
  cloudsRotation: number;
  farPlane: number;
  fogColor: TRendererColor;
  fogDensity: number;
  fogDistance: number;
  hemiColor: readonly [number, number, number, number];
  sunColor: TRendererColor;
  ambientColor: TRendererColor;
  /** The way sunlight travels, in engine space; none on an engine that stands the sun by its table. */
  sunDirection: Nullable<TRendererVector>;
  /** Radians, what the astronomical sun is turned by. */
  sunAzimuth: number;
  waterIntensity: number;
  treeAmplitude: number;
  treeSpeed: number;
  treeRotation: number;
  treeWave: TRendererVector;
}
