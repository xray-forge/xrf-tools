import { IRendererThunder, IRendererThunderboltGradient } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import {
  LevelTextureReference,
  LevelThunderbolt,
  LevelThunderboltGradient,
  LevelThunderboltModel,
  LevelThunderbolts,
} from "@/core/ipc/types/xrf-app";
import { ThunderboltCollection } from "@/core/ipc/types/xrf-environment";
import { toLevelRendererLightAnimator } from "@/core/level/lib/render/level-render-lights";
import { toRendererDraw } from "@/core/render/lib/surface/renderer-surface-draw";

/**
 * @param thunderbolts - What the game's weather strikes with.
 * @returns The same for the renderer, by texture reference; null for a game that says nowhere bolts strike.
 */
export function toLevelRendererThunder(thunderbolts: LevelThunderbolts): Nullable<IRendererThunder> {
  const { settings } = thunderbolts;

  if (!settings) {
    return null;
  }

  return {
    animators: thunderbolts.animators.map(toLevelRendererLightAnimator),
    bolts: Object.fromEntries(
      thunderbolts.bolts.map((bolt: LevelThunderbolt) => [
        bolt.name,
        {
          center: toGradient(bolt.center),
          color: bolt.color,
          model: bolt.model,
          top: toGradient(bolt.top),
        },
      ])
    ),
    collections: Object.fromEntries(
      thunderbolts.collections.map((collection: ThunderboltCollection) => [collection.name, collection.thunderbolts])
    ),
    models: thunderbolts.models.map((model: LevelThunderboltModel) => ({
      draw: toRendererDraw(model.draw),
      indices: model.mesh.indices,
      positions: model.mesh.positions.map((it: Nullable<number>) => it ?? 0),
      texture: model.mesh.texture.reference,
      uvs: model.mesh.uvs.map((it: Nullable<number>) => it ?? 0),
    })),
    settings: {
      altitude: [settings.altitude[0] ?? 0, settings.altitude[1] ?? 0],
      deltaLongitude: settings.deltaLongitude ?? 0,
      fogColor: settings.fogColor ?? 0,
      minDistance: settings.minDistance ?? 0,
      secondProbability: settings.secondProbability ?? 0,
      skyColor: settings.skyColor ?? 0,
      sunColor: settings.sunColor ?? 0,
      tilt: settings.tilt ?? 0,
    },
  };
}

/**
 * @param thunderbolts - What the game's weather strikes with.
 * @returns The textures its models and glows draw with.
 */
export function listLevelThunderTextures(thunderbolts: LevelThunderbolts): Array<LevelTextureReference> {
  return [
    ...thunderbolts.models.map((model: LevelThunderboltModel) => model.mesh.texture),
    ...thunderbolts.bolts.flatMap((bolt: LevelThunderbolt) => [bolt.top.texture, bolt.center.texture]),
  ];
}

function toGradient(gradient: LevelThunderboltGradient): IRendererThunderboltGradient {
  return {
    draw: toRendererDraw(gradient.draw),
    opacity: gradient.opacity ?? 0,
    radius: [gradient.radius[0] ?? 0, gradient.radius[1] ?? 0],
    texture: gradient.texture.reference,
  };
}
