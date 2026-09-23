import { Maybe } from "@xrf/types";
import { Material } from "three/webgpu";

import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { RENDERER_PASSES } from "#/scene/pass-record";
import { HIDDEN_MATERIAL } from "#/scene/surface/hidden-material";

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
    return material === HIDDEN_MATERIAL || Boolean(this.layouts.get(material)?.has(layout));
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
   * @returns Whether every material it draws is compiled for its layout.
   */
  public isStateReady(state: ISceneObjectState): boolean {
    return RENDERER_PASSES.every((pass) =>
      state.slots[pass].every((material: Material) => this.isReady(material, state.layout))
    );
  }
}
