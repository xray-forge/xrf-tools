import { Maybe } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IRendererSurfaceTextures } from "#/contract/scene/renderer-surface";
import { ESurfaceSlot, getSurfaceSlotPlaceholder, SURFACE_SLOTS, TSurfaceSlotTargets } from "#/material/surface-slot";
import { RendererTextures } from "#/texture/renderer-textures";
import { ITextureTarget } from "#/texture/texture-target";

/**
 * One material's slots, each drawing its key's texture once that is on the GPU and its placeholder until then.
 */
export class SurfaceSlots {
  public readonly targets: TSurfaceSlotTargets;

  private readonly textures: RendererTextures;
  private readonly bound: Array<[ESurfaceSlot, string, ITextureTarget]> = [];

  /**
   * @param textures - Where the textures are bound from.
   * @param keys - The surface's texture keys.
   * @param sampled - The slots its shader samples: the others stay at their placeholders and nothing waits for them.
   */
  public constructor(textures: RendererTextures, keys: IRendererSurfaceTextures, sampled: ReadonlyArray<ESurfaceSlot>) {
    this.textures = textures;
    this.targets = Object.fromEntries(
      SURFACE_SLOTS.map((slot: ESurfaceSlot) => {
        const placeholder: Texture = getSurfaceSlotPlaceholder(slot);
        const target: ITextureTarget = { value: placeholder };
        const key: Maybe<string> = sampled.includes(slot) ? keys[slot] : undefined;

        if (key) {
          textures.target(key, placeholder, target);
          this.bound.push([slot, key, target]);
        }

        return [slot, target];
      })
    ) as TSurfaceSlotTargets;
  }

  /** The texture keys bound, which have to be uploaded before the material draws without a stall. */
  public get keys(): Array<string> {
    return this.bound.map(([, key]) => key);
  }

  /**
   * @param slot - A slot.
   * @returns The keys bound to it: none, or the one.
   */
  public keysOf(slot: ESurfaceSlot): Array<string> {
    return this.bound.filter(([bound]) => bound === slot).map(([, key]) => key);
  }

  /** Lets every binding go, for a material that is going away. */
  public release(): void {
    this.bound.forEach(([, key, target]) => this.textures.unbind(key, target));
    this.bound.length = 0;
  }
}
