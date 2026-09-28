import { Nullable } from "@xrf/types";
import { Material } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneObjectDraw } from "#/scene/object/scene-object-draw";
import { ISceneObjectState, toSurfaceDraw } from "#/scene/object/scene-object-state";

/** A material a surface draws with, and the draw it compiles for. */
export interface ISurfacePipeline {
  material: Material;
  draw: ISceneObjectDraw;
  /** The pass drawing it, against whose target it compiles, or null for the shadow maps. */
  pass: Nullable<ERendererPass>;
}

/**
 * @param state - What an object draws.
 * @param surface - One of its surfaces.
 * @returns Every material the surface draws the object's parts with, each over its draw: its own, and for a static draw
 *   its plain pair too, which a part refused a static draw falls back to.
 */
export function toSurfacePipelines(state: ISceneObjectState, surface: ISurfaceMaterial): Array<ISurfacePipeline> {
  const draw: ISceneObjectDraw = toSurfaceDraw(state, surface);
  const pipelines: Array<ISurfacePipeline> = [{ draw, material: surface.material, pass: surface.pass }];

  if (surface.shadow) {
    pipelines.push({ draw, material: surface.shadow, pass: null });
  }

  // An impostor refused a static draw is not drawn.
  if (draw !== state.plain && !surface.isImpostor) {
    pipelines.push({ draw: state.plain, material: surface.plain.material, pass: surface.pass });

    if (surface.plain.shadow) {
      pipelines.push({ draw: state.plain, material: surface.plain.shadow, pass: null });
    }
  }

  return pipelines;
}
