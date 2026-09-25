import { Maybe } from "@xrf/types";
import { Material } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneObjectState, toSurfaceDraw } from "#/scene/object/scene-object-state";

/**
 * The vertex layouts each material's pipelines exist for, so drawing it in one stalls nothing.
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
   * @returns Whether every material it draws is compiled for its layout, the shadow materials among them.
   */
  public isStateReady(state: ISceneObjectState): boolean {
    return state.surfaces.every((surface: Maybe<ISurfaceMaterial>) => {
      if (!surface) {
        return true;
      }

      const { layout } = toSurfaceDraw(state, surface);

      return this.isReady(surface.material, layout) && (!surface.shadow || this.isReady(surface.shadow, layout));
    });
  }
}
