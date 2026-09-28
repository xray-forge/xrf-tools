import { Maybe, Nullable } from "@xrf/types";

import { LightShadowAtlas } from "#/scene/lights/light-shadow-atlas";
import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import { ILightShadowSlot } from "#/scene/lights/light-shadow-slot";
import { ILightShadowTile } from "#/scene/lights/light-shadow-tile";

/**
 * Every shadowed light's slot, by the light, and the atlas squares their faces hold: room is made from the lights out
 * of view longest, and only where what they hold makes enough.
 */
export class LightShadowSlots {
  public readonly atlas: LightShadowAtlas;

  private readonly slots: Map<number, ILightShadowSlot> = new Map();

  /**
   * @param atlas - What the faces' squares are taken from.
   */
  public constructor(atlas: LightShadowAtlas) {
    this.atlas = atlas;
  }

  /**
   * @param index - A light, as the scene's lights number it.
   * @returns Its slot, or undefined for one holding nothing.
   */
  public get(index: number): Maybe<ILightShadowSlot> {
    return this.slots.get(index);
  }

  /**
   * @param index - A light, as the scene's lights number it.
   * @returns Its slot, made empty where it had none.
   */
  public take(index: number): ILightShadowSlot {
    let slot: Maybe<ILightShadowSlot> = this.slots.get(index);

    if (!slot) {
      slot = { next: null, shown: null };
      this.slots.set(index, slot);
    }

    return slot;
  }

  /**
   * @param visit - Told every slot.
   */
  public forEach(visit: (slot: ILightShadowSlot) => void): void {
    this.slots.forEach((slot: ILightShadowSlot) => visit(slot));
  }

  /**
   * @param visit - Told every entry, shown or next.
   */
  public forEachEntry(visit: (entry: ILightShadowEntry) => void): void {
    this.slots.forEach(({ shown, next }: ILightShadowSlot) => {
      if (shown) {
        visit(shown);
      }

      if (next) {
        visit(next);
      }
    });
  }

  /**
   * Squares for a light's faces, room made from the lights not seen this frame, the longest out of view first, only
   * where it makes enough: a light evicted for nothing would come back into view dark.
   *
   * @param count - Faces.
   * @param size - Texels each is across.
   * @param frame - This frame, which the lights in view were seen in.
   * @returns The squares, or null where there is no room to make.
   */
  public allocate(count: number, size: number, frame: number): Nullable<Array<ILightShadowTile>> {
    if (this.atlas.capacity(size) < count) {
      const evicted: Nullable<Array<number>> = this.toEvicted(count, size, frame);

      if (!evicted) {
        return null;
      }

      evicted.forEach((index: number) => this.evict(index));
    }

    return Array.from({ length: count }, () => this.atlas.allocate(size) as ILightShadowTile);
  }

  /**
   * @param entry - An entry let go of, whose squares go back to the atlas.
   */
  public release(entry: Nullable<ILightShadowEntry>): void {
    entry?.faces.forEach((face: ILightShadowFace) => this.atlas.release(face.tile));
  }

  /** Forgets every slot and gives every square back. */
  public clear(): void {
    this.slots.clear();
    this.atlas.clear();
  }

  /** The fewest slots, out of view longest first, whose squares let the atlas hold `count` more of `size`. */
  private toEvicted(count: number, size: number, frame: number): Nullable<Array<number>> {
    const trial: LightShadowAtlas = this.atlas.clone();
    const evictable: Array<[number, ILightShadowSlot]> = [...this.slots]
      .filter(([, slot]: [number, ILightShadowSlot]) => isEvictable(slot, frame))
      .sort(([, a]: [number, ILightShadowSlot], [, b]: [number, ILightShadowSlot]) => toSlotSeen(a) - toSlotSeen(b));

    for (let at: number = 0; at < evictable.length; at += 1) {
      const { shown, next }: ILightShadowSlot = evictable[at][1];

      [shown, next].forEach((entry: Nullable<ILightShadowEntry>) =>
        entry?.faces.forEach((face: ILightShadowFace) => trial.release(face.tile))
      );

      if (trial.capacity(size) >= count) {
        return evictable.slice(0, at + 1).map(([index]: [number, ILightShadowSlot]) => index);
      }
    }

    return null;
  }

  private evict(index: number): void {
    const slot: Maybe<ILightShadowSlot> = this.slots.get(index);

    if (slot) {
      this.release(slot.shown);
      this.release(slot.next);
      this.slots.delete(index);
    }
  }
}

/** Whether a slot holds squares no light in view this frame is shown by. */
function isEvictable({ shown, next }: ILightShadowSlot, frame: number): boolean {
  return (shown !== null || next !== null) && (shown?.seen ?? -1) < frame && (next?.seen ?? -1) < frame;
}

function toSlotSeen({ shown, next }: ILightShadowSlot): number {
  return Math.max(shown?.seen ?? -1, next?.seen ?? -1);
}
