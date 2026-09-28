import { Maybe } from "@xrf/types";
import { Material } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { ISurfacePipeline, toSurfacePipelines } from "#/scene/surface/surface-pipeline";

/**
 * The vertex layouts each material's pipelines exist for, against the target of the pass drawing it, so drawing it in
 * one stalls nothing.
 */
export class MaterialReadiness {
  private readonly layouts: WeakMap<Material, Set<string>> = new WeakMap();

  /**
   * @param material - A material that compiled.
   * @param layout - The layout it compiled for.
   */
  public mark(material: Material, layout: string): void {
    let layouts: Maybe<Set<string>> = this.layouts.get(material);

    if (!layouts) {
      layouts = new Set();
      this.layouts.set(material, layouts);
    }

    layouts.add(layout);
  }

  /**
   * @param material - A material.
   * @param layout - A vertex layout.
   * @returns Whether drawing it with that layout needs no compile.
   */
  public isReady(material: Material, layout: string): boolean {
    return Boolean(this.layouts.get(material)?.has(layout));
  }

  /**
   * @param material - A material.
   * @returns Whether it compiled for any layout, which makes it worth keeping once nothing draws it.
   */
  public isCompiled(material: Material): boolean {
    return this.layouts.has(material);
  }

  /**
   * @param state - What an object is about to draw.
   * @param passes - The passes the frame draws or is joining: a material drawn by another waits for its pass to join.
   * @returns Whether every material it draws in them is compiled for its layout, the shadow materials and the plain
   *   fallbacks among them.
   */
  public isStateReady(state: ISceneObjectState, passes: ReadonlySet<ERendererPass>): boolean {
    return state.surfaces.every(
      (surface: Maybe<ISurfaceMaterial>) =>
        !surface ||
        toSurfacePipelines(state, surface).every(
          ({ draw, material, pass }: ISurfacePipeline) =>
            (pass !== null && !passes.has(pass)) || this.isReady(material, draw.layout)
        )
    );
  }
}
