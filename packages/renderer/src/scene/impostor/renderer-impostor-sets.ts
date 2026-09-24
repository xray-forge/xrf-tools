import { Maybe, Nullable } from "@xrf/types";

import { IRendererImpostors } from "#/contract/scene/renderer-impostors";
import { StaticDraws } from "#/scene/static/static-draws";

/** A set a consumer put, and where its impostors sit in the static draw buffers. */
interface IImpostorSet {
  count: number;
  /** Where its run of impostors starts, or null where the device's limit left no room for it. */
  start: Nullable<number>;
}

/**
 * The impostor sets a consumer put, by key: each a run of the LOD pool the cull decides clumps of trees by.
 */
export class RendererImpostorSets {
  private readonly sets: Map<string, IImpostorSet> = new Map();
  private readonly draws: StaticDraws;
  private readonly onReplaced: (key: string) => void;

  /**
   * @param draws - The static draws whose LOD pool the sets are written into.
   * @param onReplaced - Told when a key's set is put or released, so whatever names it can draw by it again.
   */
  public constructor(draws: StaticDraws, onReplaced: (key: string) => void) {
    this.draws = draws;
    this.onReplaced = onReplaced;
  }

  /**
   * @param key - A set's key.
   * @returns Where its impostors start in the LOD pool, or null for a key holding none or one left without room.
   */
  public getStart(key: Maybe<string>): Nullable<number> {
    return key ? (this.sets.get(key)?.start ?? null) : null;
  }

  public put(key: string, impostors: IRendererImpostors): void {
    this.free(key);

    const count: number = impostors.factors.length;
    const start: Nullable<number> = count ? this.draws.allocateLods(count) : null;

    if (start !== null) {
      this.draws.lods.write(start, impostors);
    }

    this.sets.set(key, { count, start });
    this.onReplaced(key);
  }

  public release(key: string): void {
    this.free(key);
    this.sets.delete(key);
    this.onReplaced(key);
  }

  public dispose(): void {
    this.sets.clear();
  }

  private free(key: string): void {
    const set: Maybe<IImpostorSet> = this.sets.get(key);

    if (set?.start !== null && set?.start !== undefined) {
      this.draws.lods.free(set.start, set.count);
    }
  }
}
