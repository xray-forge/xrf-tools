import {
  ERendererTextureEncoding,
  ERendererWeatherEngine,
  IRendererWeather,
  IRendererWeatherKeyframe,
  TRendererVector,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelTextureReference, LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { SunPosition, WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureRequests } from "@/core/level/lib/render/level-render-protocol";
import { LevelTextureReader } from "@/core/level/lib/texture/level-texture-reader";

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
 * @returns What the renderer plays, with where each sky that resolved is fetched from.
 */
export async function toLevelRendererWeather(input: ILevelRendererWeatherInput): Promise<IRendererWeather> {
  const { description, cycle, roots } = input;
  const located: Array<LevelTextureReference & { logicalPath: string }> = cycle.textures.filter(
    (it: LevelTextureReference): it is LevelTextureReference & { logicalPath: string } => it.logicalPath !== null
  );
  const requests: Array<ILevelTextureRequests> = await Promise.all(
    located.map((it) => LevelTextureReader.request(roots, it.logicalPath))
  );

  return {
    engine:
      description.engine === EXrayEngine.EXTENDED ? ERendererWeatherEngine.EXTENDED : ERendererWeatherEngine.VANILLA,
    keyframes: cycle.keyframes.map(toLevelRendererWeatherKeyframe),
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

function toTriple(value: readonly [Nullable<number>, Nullable<number>, Nullable<number>]): TRendererVector {
  return [value[0] ?? 0, value[1] ?? 0, value[2] ?? 0];
}
