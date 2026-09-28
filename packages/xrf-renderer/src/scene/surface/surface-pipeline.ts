import { Material } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneObjectDraw } from "#/scene/object/scene-object-draw";
import { ISceneObjectState, toSurfaceDraw } from "#/scene/object/scene-object-state";

/** A material a surface draws with, and the draw it compiles for. */
export interface ISurfacePipeline {
  material: Material;
  draw: ISceneObjectDraw;
  /** Whether it draws into the shadow maps rather than its pass. */
  isShadow: boolean;
}

/**
 * @param state - What an object draws.
 * @param surface - One of its surfaces.
 * @returns Every material the surface draws the object's parts with, each over its draw: its own, and for a static draw
 *   its plain pair too, which a part refused a static draw falls back to.
 */
export function toSurfacePipelines(state: ISceneObjectState, surface: ISurfaceMaterial): Array<ISurfacePipeline> {
  const draw: ISceneObjectDraw = toSurfaceDraw(state, surface);
  const pipelines: Array<ISurfacePipeline> = [{ draw, isShadow: false, material: surface.material }];

  if (surface.shadow) {
    pipelines.push({ draw, isShadow: true, material: surface.shadow });
  }

  // An impostor refused a static draw is not drawn.
  if (draw !== state.plain && !surface.isImpostor) {
    pipelines.push({ draw: state.plain, isShadow: false, material: surface.plain.material });

    if (surface.plain.shadow) {
      pipelines.push({ draw: state.plain, isShadow: true, material: surface.plain.shadow });
    }
  }

  return pipelines;
}
