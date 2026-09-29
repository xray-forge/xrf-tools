import {
  ERendererTextureEncoding,
  ERendererWeatherEngine,
  IRendererRain,
  IRendererWeather,
  IRendererWeatherKeyframe,
  IRendererWeatherModifier,
  TRendererVector,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelRain, LevelTextureReference, LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngine, XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { SunPosition, WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import { EnvModifier } from "@/core/ipc/types/xrf-level";
import { Vector3d } from "@/core/ipc/types/xrf-math";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureRequests } from "@/core/level/lib/render/level-render-protocol";
import { LevelTextureReader } from "@/core/level/lib/texture/level-texture-reader";
import { listLevelThunderTextures, toLevelRendererThunder } from "@/core/level/lib/weather/level-renderer-thunder";

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

  return {
    ...toLevelRendererWeatherBase(description),
    keyframes: cycle.keyframes.map(toLevelRendererWeatherKeyframe),
    sunTable:
      description.sunTable?.map((position: SunPosition) => ({
        altitude: position.altitude ?? 0,
        longitude: position.longitude ?? 0,
      })) ?? null,
    textures: await toLevelRendererTextures(roots, [
      ...[cycle, ...description.effects].flatMap((it: LevelWeatherCycle) => it.textures),
      ...listLevelRainTextures(description.rain),
      ...listLevelThunderTextures(description.thunderbolts),
    ]),
  };
}

/**
 * @param description - The open level's weather.
 * @returns What any weather of the level plays over whatever keyframes it plays: its engine, effects, the level's
 *   modifiers, its rain and its thunder.
 */
export function toLevelRendererWeatherBase(
  description: LevelWeatherDescription
): Pick<IRendererWeather, "engine" | "effects" | "modifiers" | "rain" | "thunder"> {
  return {
    effects: Object.fromEntries(
      description.effects.map((effect: LevelWeatherCycle) => [
        effect.name,
        effect.keyframes.map(toLevelRendererWeatherKeyframe),
      ])
    ),
    engine: toLevelRendererWeatherEngine(description.engine),
    modifiers: description.modifiers.map(toLevelRendererWeatherModifier),
    rain: toLevelRendererRain(description.rain),
    thunder: toLevelRendererThunder(description.thunderbolts),
  };
}

/**
 * @param engine - The engine target configs are read as.
 * @returns The same, as the renderer names it.
 */
export function toLevelRendererWeatherEngine(engine: XrayEngine): ERendererWeatherEngine {
  return engine === EXrayEngine.EXTENDED ? ERendererWeatherEngine.EXTENDED : ERendererWeatherEngine.VANILLA;
}

/**
 * @param rain - What the level's rain is drawn with.
 * @returns The textures it names.
 */
export function listLevelRainTextures(rain: LevelRain): Array<LevelTextureReference> {
  return [rain.streak, ...(rain.drop ? [rain.drop.texture] : [])];
}

/**
 * @param roots - Roots the level was opened in.
 * @param references - Textures and what they resolved to, each once by reference however often it is named.
 * @returns Where the renderer fetches each that resolved.
 */
export async function toLevelRendererTextures(
  roots: XrayRoots,
  references: ReadonlyArray<LevelTextureReference>
): Promise<IRendererWeather["textures"]> {
  const located: Array<LevelTextureReference & { logicalPath: string }> = [
    ...new Map(
      references
        .filter(
          (it: LevelTextureReference): it is LevelTextureReference & { logicalPath: string } => it.logicalPath !== null
        )
        .map((it) => [it.reference, it] as const)
    ).values(),
  ];
  const requests: Array<ILevelTextureRequests> = await Promise.all(
    located.map((it) => LevelTextureReader.request(roots, it.logicalPath))
  );

  return Object.fromEntries(
    located.map((it, index: number) => [
      it.reference,
      { encoding: ERendererTextureEncoding.FETCH, file: requests[index].file, picture: requests[index].picture },
    ])
  );
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
    rainColor: toTriple(keyframe.rainColor),
    rainDensity: keyframe.rainDensity ?? 0,
    thunderboltCollection: keyframe.thunderboltCollection,
    thunderboltDuration: keyframe.thunderboltDuration ?? 0,
    thunderboltPeriod: keyframe.thunderboltPeriod ?? 0,
    waterIntensity: keyframe.waterIntensity ?? 0,
    windDirection: keyframe.windDirection ?? 0,
    windVelocity: keyframe.windVelocity ?? 0,
  };
}

/**
 * @param rain - What the level's rain is drawn with.
 * @returns The same for the renderer, by texture reference, the splash's mesh as it is.
 */
export function toLevelRendererRain(rain: LevelRain): IRendererRain {
  const { drop } = rain;

  return {
    drop: drop
      ? {
          indices: drop.indices,
          positions: drop.positions.map((it: Nullable<number>) => it ?? 0),
          texture: drop.texture.reference,
          uvs: drop.uvs.map((it: Nullable<number>) => it ?? 0),
        }
      : null,
    streak: rain.streak.reference,
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
