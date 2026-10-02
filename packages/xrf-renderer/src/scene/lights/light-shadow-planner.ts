import { Maybe, Nullable } from "@xrf/types";
import { Vector3 } from "three/webgpu";

import { ILightShadowAsk } from "#/scene/lights/light-shadow-ask";
import { LIGHT_SHADOW_ATLAS_SIZE, LightShadowAtlas } from "#/scene/lights/light-shadow-atlas";
import { LightShadowChangeTracker } from "#/scene/lights/light-shadow-change-tracker";
import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import { LightShadowFaceFactory } from "#/scene/lights/light-shadow-face-factory";
import { toLightShadowFaceCount } from "#/scene/lights/light-shadow-faces";
import { ILightShadowRequest } from "#/scene/lights/light-shadow-request";
import {
  fitLightShadowScale,
  LIGHT_SHADOW_MIN_SIZE,
  toLightShadowSize,
  toLightShadowTileSize,
} from "#/scene/lights/light-shadow-sizing";
import { ILightShadowSlot } from "#/scene/lights/light-shadow-slot";
import { LightShadowSlots } from "#/scene/lights/light-shadow-slots";
import { ILightShadowTile } from "#/scene/lights/light-shadow-tile";
import { byDistance, takeSorted } from "#/scene/lights/light-sorting";
import { EShadowCasterMotion } from "#/scene/static/shadow-caster-motion";
import { SHADOW_SWAY_INTERVAL } from "#/scene/static/shadow-sway-interval";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

/** How urgently a face is drawn: what it wants, then within that, which first. */
enum EFaceUrgency {
  /** Never drawn: its light lights without its shadow until it is. */
  UNDRAWN = 0,
  /** Out of date where something changed. */
  STALE = 1,
  /** Over a caster that moves, the ones drawn longest ago first. */
  MOVING = 2,
  /** Over a caster that sways in the wind and nothing that moves: each once an interval, the longest ago first. */
  SWAYING = 3,
}

/** What owed draws are rounded up by: differences of seconds come out a hair below the whole they add to. */
const SWAY_OWED_ROUNDING: number = 1e-6;

/** A face wanting a draw this frame. */
interface IFaceCandidate {
  face: ILightShadowFace;
  urgency: EFaceUrgency;
  /** Lower first within its urgency: the light's distance, or the frame the face was drawn in. */
  order: number;
}

/**
 * Plans the lights' shadows: a square of the atlas a face, sized as the engine sizes its maps, drawn once and kept
 * while nothing it casts from changes, a few faces a frame. A face over what sways alone is drawn again once an
 * interval, those faces spread evenly over the frames between. A light lights only once its faces are drawn; one asked
 * at another size keeps its old faces until its new ones are. Room is made from the lights out of view longest; while
 * the lights in view want more than the atlas holds, every face is asked for smaller.
 */
export class LightShadowPlanner {
  public readonly atlas: LightShadowAtlas = new LightShadowAtlas(LIGHT_SHADOW_ATLAS_SIZE);
  /** The faces to draw this frame, in order. */
  public readonly queue: Array<ILightShadowFace> = [];
  /** What every wanted size is scaled by, below one while the atlas is short of room. */
  public sizeScale: number = 1;

  private readonly slots: LightShadowSlots = new LightShadowSlots(this.atlas);
  private readonly tracker: LightShadowChangeTracker;
  private readonly factory: LightShadowFaceFactory = new LightShadowFaceFactory();
  /** This frame's requests, the first `askCount` of them, their objects kept for the next frame's. */
  private readonly asks: Array<ILightShadowAsk> = [];
  private askCount: number = 0;
  private readonly candidates: Array<IFaceCandidate> = [];
  /** This frame's asks, nearest first, and its candidates, most urgent first: sorted in arrays kept between frames. */
  private readonly orderedAsks: Array<ILightShadowAsk> = [];
  private readonly orderedCandidates: Array<IFaceCandidate> = [];
  private candidateCount: number = 0;
  private frame: number = 0;
  private isWindy: boolean = false;
  /** The last frame's time, and the seconds since, at most an interval. */
  private time: Nullable<number> = null;
  private elapsed: number = 0;
  /** The swaying faces owed a draw: grown by the time passed, spent a face at a time. */
  private swayOwed: number = 0;

  /**
   * @param changes - Where what the shadow views draw changed, and what sways or moves.
   */
  public constructor(changes: StaticShadowChanges) {
    this.tracker = new LightShadowChangeTracker(changes);
  }

  /** Forgets every light's faces, for lights put again. */
  public reset(): void {
    this.slots.clear();
    this.queue.length = 0;
    this.askCount = 0;
    this.sizeScale = 1;
  }

  /** Keeps every face's square, but has each drawn again: the atlas lost what it held. */
  public forgetDrawn(): void {
    this.slots.forEachEntry((entry: ILightShadowEntry) =>
      entry.faces.forEach((face: ILightShadowFace) => (face.isDrawn = false))
    );
  }

  /**
   * Brings the faces up to what changed since the last frame.
   *
   * @param isWindy - Whether the wind sways the trees this frame.
   * @param time - Seconds, which the sway runs by.
   */
  public begin(isWindy: boolean, time: number): void {
    this.frame += 1;
    this.isWindy = isWindy;
    this.elapsed = Math.min(Math.max(time - (this.time ?? time), 0), SHADOW_SWAY_INTERVAL);
    this.time = time;
    this.askCount = 0;
    this.queue.length = 0;
    this.tracker.update(this.slots);
  }

  /**
   * Takes a shadowed light in view this frame; its faces are decided with the rest in `finish`.
   *
   * @param index - The light, as the scene's lights number it.
   * @param request - Where it stands and how it wants its faces.
   */
  public request(index: number, request: ILightShadowRequest): void {
    const ask: ILightShadowAsk = (this.asks[this.askCount] ??= {
      cone: 0,
      direction: new Vector3(),
      distance: 0,
      duel: 1,
      engineSize: 0,
      index: 0,
      intensity: 0,
      isSpot: false,
      near: 0,
      position: new Vector3(),
      range: 0,
      up: new Vector3(),
    });

    this.askCount += 1;
    ask.index = index;
    ask.isSpot = request.isSpot;
    ask.position.copy(request.position);
    ask.direction.copy(request.direction);
    ask.up.copy(request.up);
    ask.cone = request.cone;
    ask.range = request.range;
    ask.near = request.near;
    ask.intensity = request.intensity;
    ask.distance = request.distance;
    ask.duel = request.duel;
    ask.engineSize = toLightShadowSize(request);
  }

  /**
   * Gives every light asked for this frame its faces, the nearest first, queues the faces drawn this frame, and scales
   * the next frame's asks to what the atlas holds. Room is made only from the lights not asked for this frame.
   *
   * @param budget - Faces drawn at most.
   */
  public finish(budget: number): void {
    const asks: Array<ILightShadowAsk> = takeSorted(this.asks, this.askCount, this.orderedAsks, byDistance);
    const isTight: boolean = this.sizeScale < 1;
    let refused: number = 0;

    this.candidateCount = 0;

    // Every light asked for is in view: none of them is room for another.
    for (const { index } of asks) {
      const slot: Maybe<ILightShadowSlot> = this.slots.get(index);

      if (slot?.shown) {
        slot.shown.seen = this.frame;
      }

      if (slot?.next) {
        slot.next.seen = this.frame;
      }
    }

    for (const ask of asks) {
      const slot: ILightShadowSlot = this.slots.take(ask.index);
      const asked: number = toLightShadowTileSize(
        ask.engineSize * this.sizeScale,
        (slot.next ?? slot.shown)?.asked ?? 0,
        isTight
      );

      // Back at the size it shows while another was being drawn: the shown faces stand, and the other goes.
      if (slot.next && slot.shown?.asked === asked) {
        this.slots.release(slot.next);
        slot.next = null;
      }

      const current: Nullable<ILightShadowEntry> = slot.next ?? slot.shown;
      const made: Nullable<ILightShadowEntry> = current?.asked === asked ? current : this.replace(ask, slot, asked);

      // Given less than it asked, or nothing: the asks want more than the atlas holds.
      if (!made || made.size < made.asked) {
        refused += 1;
      }

      (slot.next ?? slot.shown)?.faces.forEach((face: ILightShadowFace) => this.addCandidate(face, ask.distance));
    }

    this.fillQueue(budget);
    this.slots.forEach((slot: ILightShadowSlot) => this.promote(slot));
    this.sizeScale = fitLightShadowScale(this.sizeScale, asks, refused);
  }

  /**
   * @param index - A light, as the scene's lights number it.
   * @returns Its faces as this frame shows them, every one drawn, or null where it has none whole.
   */
  public getEntry(index: number): Nullable<ILightShadowEntry> {
    const shown: Nullable<ILightShadowEntry> = this.slots.get(index)?.shown ?? null;

    return shown && shown.seen === this.frame && this.isReady(shown) ? shown : null;
  }

  /** Marks the queue drawn and current. */
  public markDrawn(): void {
    for (const face of this.queue) {
      face.isDrawn = true;
      face.isStale = false;
      face.drawnAt = this.frame;
    }
  }

  /** Whether every face of an entry is drawn, or is queued to be this frame. */
  private isReady(entry: ILightShadowEntry): boolean {
    return entry.faces.every((face: ILightShadowFace) => face.isDrawn || this.queue.includes(face));
  }

  /**
   * Makes a light's faces at a new size: drawn beside the shown ones while those are whole, so it never goes dark, and
   * in their place otherwise.
   *
   * @returns The faces made, or null where the atlas had no room.
   */
  private replace(ask: ILightShadowAsk, slot: ILightShadowSlot, asked: number): Nullable<ILightShadowEntry> {
    this.slots.release(slot.next);
    slot.next = null;

    if (slot.shown && !this.isReady(slot.shown)) {
      this.slots.release(slot.shown);
      slot.shown = null;
    }

    let entry: Nullable<ILightShadowEntry> = this.create(ask, asked);

    // No room beside the shown faces: the new ones take theirs, and the light waits for them.
    if (!entry && slot.shown) {
      this.slots.release(slot.shown);
      slot.shown = null;
      entry = this.create(ask, asked);
    }

    if (slot.shown) {
      slot.next = entry;
    } else {
      slot.shown = entry;
    }

    return entry;
  }

  /** Shows a slot's replacement once every face of it is drawn, letting the old faces go. */
  private promote(slot: ILightShadowSlot): void {
    if (slot.next && this.isReady(slot.next)) {
      this.slots.release(slot.shown);
      slot.shown = slot.next;
      slot.next = null;
    }
  }

  /** A light's faces at the size asked, or the largest smaller that has room, or null for none. */
  private create(ask: ILightShadowAsk, asked: number): Nullable<ILightShadowEntry> {
    const count: number = toLightShadowFaceCount(ask.isSpot);

    for (let size: number = asked; size >= LIGHT_SHADOW_MIN_SIZE; size /= 2) {
      const tiles: Nullable<Array<ILightShadowTile>> = this.slots.allocate(count, size, this.frame);

      if (tiles) {
        const entry: ILightShadowEntry = this.factory.create(ask, asked, tiles, this.frame);

        this.tracker.findMotion(entry);

        return entry;
      }
    }

    return null;
  }

  private addCandidate(face: ILightShadowFace, distance: number): void {
    let urgency: EFaceUrgency;

    if (!face.isDrawn) {
      urgency = EFaceUrgency.UNDRAWN;
    } else if (face.isStale) {
      urgency = EFaceUrgency.STALE;
    } else if (face.motion === EShadowCasterMotion.MOVING) {
      urgency = EFaceUrgency.MOVING;
    } else if (face.motion === EShadowCasterMotion.SWAYING && this.isWindy) {
      urgency = EFaceUrgency.SWAYING;
    } else {
      return;
    }

    const candidate: IFaceCandidate = (this.candidates[this.candidateCount] ??= {
      face,
      order: 0,
      urgency: EFaceUrgency.UNDRAWN,
    });

    this.candidateCount += 1;
    candidate.face = face;
    candidate.urgency = urgency;
    candidate.order = urgency >= EFaceUrgency.MOVING ? face.drawnAt : distance;
  }

  private fillQueue(budget: number): void {
    const candidates: Array<IFaceCandidate> = takeSorted(
      this.candidates,
      this.candidateCount,
      this.orderedCandidates,
      byUrgency
    );

    let swaying: number = 0;

    for (const { urgency } of candidates) {
      swaying += urgency === EFaceUrgency.SWAYING ? 1 : 0;
    }

    // Each once an interval, a share a frame: a steady cost however fast the frames come, and never more than all.
    this.swayOwed = swaying ? Math.min(this.swayOwed + (swaying * this.elapsed) / SHADOW_SWAY_INTERVAL, swaying) : 0;

    for (const { face, urgency } of candidates) {
      if (this.queue.length === budget) {
        break;
      }

      if (urgency !== EFaceUrgency.SWAYING) {
        this.queue.push(face);
      } else if (this.swayOwed + SWAY_OWED_ROUNDING >= 1) {
        this.queue.push(face);
        this.swayOwed -= 1;
      }
    }
  }
}

function byUrgency(a: IFaceCandidate, b: IFaceCandidate): number {
  return a.urgency - b.urgency || a.order - b.order;
}
