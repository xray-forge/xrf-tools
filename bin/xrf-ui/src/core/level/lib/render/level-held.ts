import { Nullable } from "@xrf/types";

/** Told a held value, or null for none. */
export type TLevelHeldListener<T> = (value: Nullable<T>) => void;

/**
 * Something a level reads once it opens and hands to whatever draws it: told as it is now and whenever it changes,
 * with the textures it is dressed with kept for as long as it is held.
 */
export class LevelHeld<T> {
  private value: Nullable<T> = null;
  private readonly watchers: Set<TLevelHeldListener<T>> = new Set();
  /** The texture references it binds, which stay uploaded while it is held. */
  private currentTextures: ReadonlySet<string> = new Set();

  public get held(): Nullable<T> {
    return this.value;
  }

  public get textures(): ReadonlySet<string> {
    return this.currentTextures;
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
   * @param value - What is held from now on, or null for nothing.
   * @param textures - The texture references it binds.
   */
  public hold(value: Nullable<T>, textures: ReadonlySet<string> = new Set()): void {
    this.value = value;
    this.currentTextures = value ? textures : new Set();

    for (const watcher of Array.from(this.watchers)) {
      watcher(value);
    }
  }
}
