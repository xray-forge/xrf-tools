import {
  ERendererTextureEncoding,
  ERendererWeatherEngine,
  IRendererWeather,
  IRendererWeatherKeyframe,
  IRendererWeatherModifier,
  TRendererVector,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelTextureReference, LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { SunPosition, WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import { EnvModifier } from "@/core/ipc/types/xrf-level";
import { Vector3d } from "@/core/ipc/types/xrf-math";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureRequests } from "@/core/level/lib/render/level-render-protocol";
import { LevelTextureReader } from "@/core/level/lib/texture/level-texture-reader";

/** Every value a modifier adds to, what a file older than the flags stands for. */
const ALL_MODIFIER_FLAGS: number = 0xffff;

/** What a weather the renderer plays is built from. */
export interface ILevelRendererWeatherInput {
  /** The open level's weather. */
  description: LevelWeatherDescription;
  /** The cycle played. */
  cycle: LevelWeatherCycle;
  /** Roots the level was opened in, which its skies are read from. */
  roots: XrayRoots;
}

/**
 * @param input - The cycle, the level's weather it belongs to and where its skies are read from.
 * @returns What the renderer plays: the cycle, every effect and the level's modifiers, with where each texture that
 *   resolved is fetched from.
 */
export async function toLevelRendererWeather(input: ILevelRendererWeatherInput): Promise<IRendererWeather> {
  const { description, cycle, roots } = input;
  const located: Array<LevelTextureReference & { logicalPath: string }> = [
    ...new Map(
      [cycle, ...description.effects]
        .flatMap((it: LevelWeatherCycle) => it.textures)
        .filter(
          (it: LevelTextureReference): it is LevelTextureReference & { logicalPath: string } => it.logicalPath !== null
        )
        .map((it) => [it.reference, it] as const)
    ).values(),
  ];
  const requests: Array<ILevelTextureRequests> = await Promise.all(
    located.map((it) => LevelTextureReader.request(roots, it.logicalPath))
  );

  return {
    engine:
      description.engine === EXrayEngine.EXTENDED ? ERendererWeatherEngine.EXTENDED : ERendererWeatherEngine.VANILLA,
    effects: Object.fromEntries(
      description.effects.map((effect: LevelWeatherCycle) => [
        effect.name,
        effect.keyframes.map(toLevelRendererWeatherKeyframe),
      ])
    ),
    keyframes: cycle.keyframes.map(toLevelRendererWeatherKeyframe),
    modifiers: description.modifiers.map(toLevelRendererWeatherModifier),
    sunTable:
      description.sunTable?.map((position: SunPosition) => ({
        altitude: position.altitude ?? 0,
        longitude: position.longitude ?? 0,
      })) ?? null,
    textures: Object.fromEntries(
      located.map((it, index: number) => [
        it.reference,
        { encoding: ERendererTextureEncoding.FETCH, file: requests[index].file, picture: requests[index].picture },
      ])
    ),
  };
}

/**
 * @param keyframe - A keyframe as the engine loads it.
 * @returns What the renderer's mixer reads of it, a number the backend could not write as zero.
 */
export function toLevelRendererWeatherKeyframe(keyframe: WeatherDescriptor): IRendererWeatherKeyframe {
  return {
    ambientColor: toTriple(keyframe.ambientColor),
    cloudsColor: [
      keyframe.cloudsColor[0] ?? 0,
      keyframe.cloudsColor[1] ?? 0,
      keyframe.cloudsColor[2] ?? 0,
      keyframe.cloudsColor[3] ?? 0,
    ],
    cloudsRotation: keyframe.cloudsRotation ?? 0,
    cloudsTexture: keyframe.cloudsTexture,
    farPlane: keyframe.farPlane ?? 0,
    fogColor: toTriple(keyframe.fogColor),
    fogDensity: keyframe.fogDensity ?? 0,
    fogDistance: keyframe.fogDistance ?? 0,
    hemiColor: [
      keyframe.hemiColor[0] ?? 0,
      keyframe.hemiColor[1] ?? 0,
      keyframe.hemiColor[2] ?? 0,
      keyframe.hemiColor[3] ?? 0,
    ],
    skyColor: toTriple(keyframe.skyColor),
    skyRotation: keyframe.skyRotation ?? 0,
    skyTexture: keyframe.skyTexture,
    skyTextureEnv: keyframe.skyTextureEnv,
    sunAzimuth: keyframe.sunAzimuth ?? 0,
    sunColor: toTriple(keyframe.sunColor),
    sunDirection: keyframe.sunDirection ? toTriple(keyframe.sunDirection) : null,
    time: keyframe.time,
    treeAmplitude: keyframe.treeAmplitude ?? 0,
    treeRotation: keyframe.treeRotation ?? 0,
    treeSpeed: keyframe.treeSpeed ?? 0,
    treeWave: toTriple(keyframe.treeWave),
    waterIntensity: keyframe.waterIntensity ?? 0,
  };
}

/**
 * @param modifier - One `level.env_mod` volume, as the level file holds it.
 * @returns What the renderer's mixer weighs of it, in engine space.
 */
export function toLevelRendererWeatherModifier(modifier: EnvModifier): IRendererWeatherModifier {
  return {
    ambient: toVector(modifier.ambient),
    farPlane: modifier.farPlane ?? 0,
    flags: modifier.useFlags ?? ALL_MODIFIER_FLAGS,
    fogColor: toVector(modifier.fogColor),
    fogDensity: modifier.fogDensity ?? 0,
    hemiColor: toVector(modifier.hemiColor),
    position: toVector(modifier.position),
    power: modifier.power ?? 0,
    radius: modifier.radius ?? 0,
    skyColor: toVector(modifier.skyColor),
  };
}

function toVector(value: Vector3d): TRendererVector {
  return [value.x ?? 0, value.y ?? 0, value.z ?? 0];
}

function toTriple(value: readonly [Nullable<number>, Nullable<number>, Nullable<number>]): TRendererVector {
  return [value[0] ?? 0, value[1] ?? 0, value[2] ?? 0];
}
