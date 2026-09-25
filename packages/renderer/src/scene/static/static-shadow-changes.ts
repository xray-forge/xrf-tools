import { Maybe, Nullable } from "@xrf/types";
import { Box3 } from "three/webgpu";

/** Changes the log keeps; one older than the oldest kept stands for a change anywhere. */
const LOG_LIMIT: number = 4096;

/** One change to what the shadow views draw: where it was, or null for anywhere. */
interface IShadowChange {
  serial: number;
  box: Nullable<Box3>;
}

/**
 * Where what the shadow views draw changed: every casting slot's box, which a slot going or coming, or a texture it
 * cuts out by being replaced, logs as changed; and the casting slots that sway in the wind. A shadow kept over a
 * region that changed is drawn again, and one kept over a swaying caster while the wind blows.
 */
export class StaticShadowChanges {
  private serial: number = 0;
  private log: Array<IShadowChange> = [];
  /** The serial before the oldest change kept: anything asked from earlier is taken as changed everywhere. */
  private dropped: number = 0;
  /** Every casting slot's box, null where it is not known. */
  private readonly casting: Map<number, Nullable<Box3>> = new Map();
  private readonly swaying: Map<number, Box3> = new Map();
  private currentSwayingVersion: number = 0;

  /** Bumped by every change logged. */
  public get version(): number {
    return this.serial;
  }

  /** Bumped whenever a swaying caster came or went. */
  public get swayingVersion(): number {
    return this.currentSwayingVersion;
  }

  /** The boxes of every casting slot the wind sways. */
  public get swayingBoxes(): Iterable<Box3> {
    return this.swaying.values();
  }

  /**
   * @param slot - A slot drawing from now on.
   * @param bounds - What it spans, null where it is not known.
   * @param isCasting - Whether its surface casts.
   * @param isSwaying - Whether the wind sways it.
   */
  public put(slot: number, bounds: Nullable<Box3>, isCasting: boolean, isSwaying: boolean): void {
    this.withdraw(slot);

    if (!isCasting) {
      return;
    }

    this.casting.set(slot, bounds);
    this.note(bounds);

    if (isSwaying && bounds) {
      this.swaying.set(slot, bounds);
      this.currentSwayingVersion += 1;
    }
  }

  /**
   * @param slot - A slot no longer drawing.
   */
  public withdraw(slot: number): void {
    if (!this.casting.has(slot)) {
      return;
    }

    this.note(this.casting.get(slot) ?? null);
    this.casting.delete(slot);

    if (this.swaying.delete(slot)) {
      this.currentSwayingVersion += 1;
    }
  }

  /**
   * @param slot - A casting slot drawn differently from now on where it stands, as a texture it cuts out by replaced.
   */
  public touch(slot: number): void {
    if (this.casting.has(slot)) {
      this.note(this.casting.get(slot) ?? null);
    }
  }

  /**
   * @param since - The version a shadow was drawn at.
   * @returns Where anything changed since, a null box for anywhere, or null where the log no longer reaches back.
   */
  public since(since: number): Nullable<ReadonlyArray<Nullable<Box3>>> {
    if (since < this.dropped) {
      return null;
    }

    const first: Maybe<IShadowChange> = this.log[0];
    const start: number = first ? Math.max(0, since - first.serial + 1) : 0;

    return this.log.slice(start).map((change: IShadowChange) => change.box);
  }

  private note(box: Nullable<Box3>): void {
    this.serial += 1;
    this.log.push({ box, serial: this.serial });

    if (this.log.length > LOG_LIMIT) {
      this.dropped = (this.log.shift() as IShadowChange).serial;
    }
  }
}
