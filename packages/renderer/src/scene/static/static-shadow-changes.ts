import { Nullable } from "@xrf/types";
import { Box3, Sphere, Vector4 } from "three/webgpu";

import { isBoxInPlanes } from "#/scene/static/static-cell";

/** Changes the log keeps; one older than the oldest kept stands for a change anywhere. */
const LOG_LIMIT: number = 4096;

/** What casts into the shadow views: a static slot by its number, or a part drawn plainly by itself. */
export type TShadowCasterKey = number | object;

/** How a caster moves while it casts, which has a kept shadow over it drawn again. */
export enum EShadowCasterMotion {
  /** Not at all: a shadow over it is drawn again only once something there changes. */
  STILL = 0,
  /** With the wind: while it blows. */
  SWAYING = 1,
  /** On its own, as a skinned part plays: always. */
  MOVING = 2,
}

/** One change to what the shadow views draw. */
export interface IShadowChange {
  /** Where it was, or null for anywhere. */
  readonly box: Nullable<Box3>;
  /** Whether what came or went there sways or moves, so a kept shadow there looks again for what does. */
  readonly isAnimated: boolean;
}

/** What changed since a version: every change kept, or anywhere, the log no longer reaching back. */
export interface IShadowChanges {
  readonly isEverywhere: boolean;
  readonly changes: ReadonlyArray<IShadowChange>;
}

/** A change as the log keeps it, its record written over as the ring comes round. */
interface IShadowChangeRecord {
  box: Nullable<Box3>;
  isAnimated: boolean;
}

/** A caster that sways or moves: its broad bound and, for a listed draw, each of its places' spheres. */
interface IAnimatedCaster {
  motion: EShadowCasterMotion;
  bounds: Nullable<Box3>;
  spheres: Nullable<Float32Array>;
}

/**
 * Where what the shadow views draw changed: every caster's box, which a caster coming, going or moving, or a texture it
 * cuts out by being replaced, logs as changed; and the casters that sway or move. A shadow kept over a region that
 * changed is drawn again, and one kept over a caster that sways or moves while it does.
 */
export class StaticShadowChanges {
  private serial: number = 0;
  /** The last changes, a ring `LOG_LIMIT` long, the change of serial `s` at `s % LOG_LIMIT`. */
  private readonly log: Array<IShadowChangeRecord> = Array.from({ length: LOG_LIMIT }, () => ({
    box: null,
    isAnimated: false,
  }));
  /** What `since` answers, written again each call: it is read at once, never kept. */
  private readonly answer: { isEverywhere: boolean; changes: Array<IShadowChange> } = {
    changes: [],
    isEverywhere: false,
  };
  /** Every caster's box, null where it is not known. */
  private readonly casting: Map<TShadowCasterKey, Nullable<Box3>> = new Map();
  private readonly animated: Map<TShadowCasterKey, IAnimatedCaster> = new Map();
  private readonly instanceBox: Box3 = new Box3();

  /** Bumped by every change logged. */
  public get version(): number {
    return this.serial;
  }

  /**
   * @param caster - What casts from now on.
   * @param bounds - What it spans, null where it is not known.
   * @param isCasting - Whether its surface casts.
   * @param motion - How it moves while it casts.
   * @param spheres - A listed draw's places' spheres in renderer space, four floats each, a negative radius for none;
   *   copied for a caster that sways or moves. Null for a single draw or unknown places.
   */
  public put(
    caster: TShadowCasterKey,
    bounds: Nullable<Box3>,
    isCasting: boolean,
    motion: EShadowCasterMotion = EShadowCasterMotion.STILL,
    spheres: Nullable<Float32Array> = null
  ): void {
    this.withdraw(caster);

    if (!isCasting) {
      return;
    }

    const isAnimated: boolean = motion !== EShadowCasterMotion.STILL;

    this.casting.set(caster, bounds);

    if (isAnimated) {
      this.animated.set(caster, { bounds, motion, spheres: spheres?.slice() ?? null });
    }

    this.note(bounds, isAnimated);
  }

  /**
   * @param caster - What casts no more.
   */
  public withdraw(caster: TShadowCasterKey): void {
    if (!this.casting.has(caster)) {
      return;
    }

    const bounds: Nullable<Box3> = this.casting.get(caster) ?? null;

    this.casting.delete(caster);
    this.note(bounds, this.animated.delete(caster));
  }

  /**
   * @param caster - A caster drawn differently from now on where it stands, as by a texture it cuts out replaced.
   */
  public touch(caster: TShadowCasterKey): void {
    if (this.casting.has(caster)) {
      this.note(this.casting.get(caster) ?? null, false);
    }
  }

  /**
   * @param since - The version a shadow was drawn at.
   * @returns What changed since.
   */
  public since(since: number): IShadowChanges {
    const oldest: number = this.serial - Math.min(this.serial, LOG_LIMIT);
    const { answer } = this;

    answer.changes.length = 0;
    answer.isEverywhere = since < oldest;

    if (answer.isEverywhere) {
      return answer;
    }

    for (let serial: number = since + 1; serial <= this.serial; serial += 1) {
      const change: IShadowChangeRecord = this.log[serial % LOG_LIMIT];

      answer.changes.push(change);
      answer.isEverywhere ||= change.box === null;
    }

    return answer;
  }

  /**
   * Finds the fastest a caster in a shadow view moves: a listed draw's own places tested, so the room between two trees
   * far apart has nothing drawn again.
   *
   * @param planes - The view's frustum planes, in renderer space.
   * @param sphere - What the view's light reaches, for a light's face; none for a cascade.
   * @returns How the fastest caster there moves; a caster of unknown bounds stands everywhere.
   */
  public getMotion(planes: ReadonlyArray<Vector4>, sphere: Nullable<Sphere> = null): EShadowCasterMotion {
    let motion: EShadowCasterMotion = EShadowCasterMotion.STILL;

    for (const caster of this.animated.values()) {
      if (caster.motion > motion && this.isInView(caster, planes, sphere)) {
        motion = caster.motion;
      }
    }

    return motion;
  }

  private isInView(caster: IAnimatedCaster, planes: ReadonlyArray<Vector4>, sphere: Nullable<Sphere>): boolean {
    const { bounds, spheres } = caster;

    if (bounds && ((sphere && !sphere.intersectsBox(bounds)) || !isBoxInPlanes(bounds, planes))) {
      return false;
    }

    if (spheres === null) {
      return true;
    }

    for (let at: number = 0; at < spheres.length; at += 4) {
      const radius: number = spheres[at + 3];

      if (radius < 0) {
        continue;
      }

      this.instanceBox.min.set(spheres[at] - radius, spheres[at + 1] - radius, spheres[at + 2] - radius);
      this.instanceBox.max.set(spheres[at] + radius, spheres[at + 1] + radius, spheres[at + 2] + radius);

      if ((!sphere || sphere.intersectsBox(this.instanceBox)) && isBoxInPlanes(this.instanceBox, planes)) {
        return true;
      }
    }

    return false;
  }

  private note(box: Nullable<Box3>, isAnimated: boolean): void {
    const record: IShadowChangeRecord = this.log[(this.serial + 1) % LOG_LIMIT];

    this.serial += 1;
    record.box = box;
    record.isAnimated = isAnimated;
  }
}
