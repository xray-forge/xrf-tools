import { TextureNode } from "three/webgpu";

import { ERendererLightKind, TRendererLight } from "#/contract/scene/renderer-lights";
import { toProjectorAnchor } from "#/scene/lights/light-projectors.tsl";
import { getWhiteTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";

/** Distinct projectors the spots of one scene sample; a spot naming another lights white. */
export const MAX_PROJECTORS: number = 8;

/**
 * The spots' projectors: a sampler a slot, bound to the texture the slot's key names, white where no spot names one.
 * The lights pass samples every slot, and builds its shader again whenever they are bound again.
 */
export class LightProjectors {
  /** A sampler a slot. */
  public samplers: ReadonlyArray<TextureNode> = [];
  /** Bumped whenever the slots are bound again. */
  public version: number = 0;

  private readonly textures: RendererTextures;
  /** The key each slot is bound to. */
  private keys: ReadonlyArray<string> = [];

  /**
   * @param textures - What the projectors' textures are bound through.
   */
  public constructor(textures: RendererTextures) {
    this.textures = textures;
    this.bind([]);
  }

  /**
   * @param lights - The scene's lights, whose spots name the projectors, the first to name one taking the first slot.
   */
  public put(lights: ReadonlyArray<TRendererLight>): void {
    const keys: Array<string> = [];

    for (const light of lights) {
      if (
        light.kind === ERendererLightKind.SPOT &&
        light.projector &&
        !keys.includes(light.projector) &&
        keys.length < MAX_PROJECTORS
      ) {
        keys.push(light.projector);
      }
    }

    this.bind(keys);
  }

  /**
   * @param light - A light.
   * @returns The slot it samples, or -1 for none: a point, a spot without a projector, or one past the slots.
   */
  public getSlot(light: TRendererLight): number {
    return light.kind === ERendererLightKind.SPOT && light.projector ? this.keys.indexOf(light.projector) : -1;
  }

  /** Points every slot at white. */
  public release(): void {
    this.bind([]);
  }

  /** Lets every slot's texture go, binding nothing in their place. */
  public dispose(): void {
    this.unbind();
    this.samplers = [];
    this.keys = [];
  }

  private bind(keys: ReadonlyArray<string>): void {
    if (this.samplers.length && keys.join() === this.keys.join()) {
      return;
    }

    this.unbind();
    this.keys = keys;
    this.samplers = Array.from({ length: MAX_PROJECTORS }, (_, slot: number) =>
      this.textures.bind(keys[slot], getWhiteTexture(), toProjectorAnchor())
    );
    this.version += 1;
  }

  private unbind(): void {
    this.samplers.forEach((sampler: TextureNode, slot: number) => this.textures.unbind(this.keys[slot], sampler));
  }
}
