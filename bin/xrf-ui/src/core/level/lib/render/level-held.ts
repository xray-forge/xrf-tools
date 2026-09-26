import { Nullable } from "@xrf/types";

import { ILevelHeldSource, TLevelHeldListener } from "@/core/level/lib/render/level-render-protocol";

/**
 * Something a level reads once it opens and hands to whatever draws it: told as it is now and whenever it changes.
 * The textures it is dressed with are claimed before it is held, so they stay uploaded while they are still being read,
 * and go with it.
 */
export class LevelHeld<T> implements ILevelHeldSource<T> {
  private value: Nullable<T> = null;
  private readonly watchers: Set<TLevelHeldListener<T>> = new Set();
  /** The texture references it binds, which stay uploaded while it is held or about to be. */
  private claimed: ReadonlySet<string> = new Set();

  public get held(): Nullable<T> {
    return this.value;
  }

  public get textures(): ReadonlySet<string> {
    return this.claimed;
  }

  /**
   * @param listener - Told the value held now, then whenever it changes.
   * @returns Stops the telling.
   */
  public subscribe(listener: TLevelHeldListener<T>): () => void {
    this.watchers.add(listener);
    listener(this.value);

    return (): void => {
      this.watchers.delete(listener);
    };
  }

  /**
   * Keeps texture references uploaded ahead of what binds them, telling nobody: nothing is held yet.
   *
   * @param textures - The references the value about to be held binds.
   */
  public claim(textures: ReadonlySet<string>): void {
    this.claimed = textures;
  }

  /**
   * @param value - What is held from now on, dressed with the textures claimed for it.
   */
  public hold(value: T): void {
    this.value = value;
    this.tell();
  }

  /** Lets the value go, and the textures claimed with it. */
  public release(): void {
    const isHeld: boolean = this.value !== null;

    this.value = null;
    this.claimed = new Set();

    if (isHeld) {
      this.tell();
    }
  }

  private tell(): void {
    for (const watcher of Array.from(this.watchers)) {
      watcher(this.value);
    }
  }
}
