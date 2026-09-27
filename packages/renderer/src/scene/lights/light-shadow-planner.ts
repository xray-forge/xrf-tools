import { Maybe, Nullable } from "@xrf/types";
import { Box3, Frustum, Matrix4, PerspectiveCamera, Sphere, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { ILightShadowTile, LightShadowAtlas } from "#/scene/lights/light-shadow-atlas";
import {
  LIGHT_SHADOW_POINT_CONE,
  LIGHT_SHADOW_POINT_FACES,
  LIGHT_SHADOW_WIDENING,
  toLightShadowFaceCount,
} from "#/scene/lights/light-shadow-faces";
import { isBoxInPlanes } from "#/scene/static/static-cell";
import {
  EShadowCasterMotion,
  IShadowChange,
  IShadowChanges,
  StaticShadowChanges,
} from "#/scene/static/static-shadow-changes";
import { toCameraFrustum, toPlaneVectors } from "#/visibility/camera-frustum";
import { IShadowFrustum } from "#/visibility/shadow-frustum";

/** Texels the atlas is across. */
export const LIGHT_SHADOW_ATLAS_SIZE: number = 4096;

/** `SMAP_adapt_min`, `SMAP_adapt_optimal` and `SMAP_adapt_max` (`r2_types.h`). */
const SMAP_MIN: number = 32;
const SMAP_OPTIMAL: number = 768;
const SMAP_MAX: number = 1536;

/** The largest square a face takes, which leaves the atlas room for the rest. */
const TILE_MAX: number = 1024;

/** How far a face's wanted size may stray from the square it was asked at before it is asked at another. */
const GROW: number = 1.5;
const SHRINK: number = 0.66;

/** The share of the atlas the lights in view may want before every face is asked for smaller. */
const FILL: number = 0.75;

/** What the wanted sizes' scale is multiplied by for a frame a light in view was refused room, and by to recover. */
const TIGHTEN: number = 0.7;
const LOOSEN: number = 1.25;

/** The least the wanted sizes are scaled to. */
const MIN_SCALE: number = 1 / 32;

/** `EPS_S`: what a face's far plane stands past the light's range. */
const FAR_EPSILON: number = 0.0000001;

/** Where a face's projection starts where the light gives none: `light::virtual_size`'s default. */
const DEFAULT_NEAR: number = 0.1;

/** The texels of the whole atlas. */
const ATLAS_AREA: number = LIGHT_SHADOW_ATLAS_SIZE * LIGHT_SHADOW_ATLAS_SIZE;

/** A shadowed light as the planner takes it, in world space; copied, so one object may ask for every light. */
export interface ILightShadowRequest {
  isSpot: boolean;
  position: Vector3;
  /** A spot's direction and up, square to each other. */
  direction: Vector3;
  up: Vector3;
  /** A spot's whole cone. */
  cone: number;
  range: number;
  near: number;
  /** `(mean + luminance) / 2` of its colour, which a brighter light earns a larger map by. */
  intensity: number;
  /** Metres from the camera to its spatial sphere's edge, none inside it. */
  distance: number;
  /** `1 - dot(camera forward, light direction) / 2`: a light facing the camera earns more. */
  duel: number;
}

/** One face of a light's shadow: its view and projection, where it is drawn in the atlas, and whether it is current. */
export interface ILightShadowFace extends IShadowFrustum {
  /** Its camera's world matrix and its inverse, the view. */
  readonly world: Matrix4;
  readonly view: Matrix4;
  readonly projection: Matrix4;
  readonly near: number;
  readonly far: number;
  readonly tile: ILightShadowTile;
  /** Whether it was drawn since its square last lost what it held. */
  isDrawn: boolean;
  /** Whether something it casts from changed since it was drawn. */
  isStale: boolean;
  /** How the fastest caster standing in it moves, which has it drawn again while it does. */
  motion: EShadowCasterMotion;
  /** The frame it was last drawn in, which the faces over what moves take turns by. */
  drawnAt: number;
}

/** A shadowed light's faces, made for the size its light asked. */
export interface ILightShadowEntry {
  readonly faces: ReadonlyArray<ILightShadowFace>;
  /** The square its faces were asked at, and the one they were given, smaller where the atlas had no room. */
  readonly asked: number;
  readonly size: number;
  readonly near: number;
  readonly far: number;
  /** Everything it reaches, which a change must touch to touch any of its faces. */
  readonly sphere: Sphere;
  /** The frame its light was last asked for. */
  seen: number;
}

/** A light's entry shown, and the one drawn to replace it at another size while that is not yet whole. */
interface ILightShadowSlot {
  shown: Nullable<ILightShadowEntry>;
  next: Nullable<ILightShadowEntry>;
}

/** A request taken this frame, copied, with the size the engine wants its faces at. */
interface ILightShadowAsk extends ILightShadowRequest {
  index: number;
  engineSize: number;
}

/** How urgently a face is drawn: what it wants, then within that, which first. */
enum EFaceUrgency {
  /** Never drawn: its light lights without its shadow until it is. */
  UNDRAWN = 0,
  /** Out of date where something changed. */
  STALE = 1,
  /** Over a caster that sways or moves, the ones drawn longest ago first. */
  ANIMATED = 2,
}

/** A face wanting a draw this frame. */
interface IFaceCandidate {
  face: ILightShadowFace;
  urgency: EFaceUrgency;
  /** Lower first within its urgency: the light's distance, or the frame the face was drawn in. */
  order: number;
}

/**
 * `compute_xf_spot`'s map size: larger for a light nearer, brighter, facing the camera, longer and wider, in texels.
 *
 * @param request - The light.
 * @returns The size the engine would draw its map at.
 */
export function toLightShadowSize(request: ILightShadowRequest): number {
  const area: number = Math.min(Math.max((request.range * request.range) / (1 + request.distance ** 2), 0), 1);
  const cone: number = request.isSpot ? request.cone : LIGHT_SHADOW_POINT_CONE;
  const factor: number =
    Math.sqrt(area) *
    Math.pow(Math.max(request.intensity, 0), 1 / 16) *
    Math.pow(request.duel, 1 / 4) *
    Math.pow(request.range / 8, 1 / 4) *
    Math.sqrt(cone / (Math.PI / 2));

  return Math.min(Math.max(Math.floor(factor * SMAP_OPTIMAL), SMAP_MIN), SMAP_MAX);
}

/**
 * @param size - The size a face is wanted at.
 * @param current - The square it was asked at, or zero for none, kept while the size stays within `SHRINK..GROW` of it.
 * @param isTight - Whether the atlas is short of room, which takes the power of two at or below the size.
 * @returns The square it is to be asked at: the power of two nearest the size.
 */
export function toLightShadowTileSize(size: number, current: number, isTight: boolean = false): number {
  if (isTight) {
    return Math.min(Math.max(2 ** Math.floor(Math.log2(Math.max(size, 1))), SMAP_MIN), TILE_MAX);
  }

  if (current > 0 && size <= current * GROW && size >= current * SHRINK) {
    return current;
  }

  return Math.min(Math.max(2 ** Math.round(Math.log2(Math.max(size, 1))), SMAP_MIN), TILE_MAX);
}

/**
 * Plans the lights' shadows: a square of the atlas a face, sized as the engine sizes its maps, drawn once and kept
 * while nothing it casts from changes, a few faces a frame. A light lights only once its faces are drawn; one asked
 * at another size keeps its old faces until its new ones are. Room is made from the lights out of view longest; while
 * the lights in view want more than the atlas holds, every face is asked for smaller.
 */
export class LightShadowPlanner {
  public readonly atlas: LightShadowAtlas = new LightShadowAtlas(LIGHT_SHADOW_ATLAS_SIZE);
  /** The faces to draw this frame, in order. */
  public readonly queue: Array<ILightShadowFace> = [];
  /** What every wanted size is scaled by, below one while the atlas is short of room. */
  public sizeScale: number = 1;

  private readonly changes: StaticShadowChanges;
  private readonly slots: Map<number, ILightShadowSlot> = new Map();
  /** The entries a change of something that sways or moves reached this frame, whose motion is found once. */
  private readonly moved: Set<ILightShadowEntry> = new Set();
  /** This frame's requests, the first `askCount` of them, their objects kept for the next frame's. */
  private readonly asks: Array<ILightShadowAsk> = [];
  private askCount: number = 0;
  private readonly candidates: Array<IFaceCandidate> = [];
  /** This frame's asks, nearest first, and its candidates, most urgent first: sorted in arrays kept between frames. */
  private readonly orderedAsks: Array<ILightShadowAsk> = [];
  private readonly orderedCandidates: Array<IFaceCandidate> = [];
  private candidateCount: number = 0;
  private frame: number = 0;
  private faceVersion: number = 0;
  /** The change log's version the faces were last brought up to. */
  private changeVersion: number;
  private isWindy: boolean = false;
  /** What a face's matrices are built with, and its frustum found by. */
  private readonly camera: PerspectiveCamera = new PerspectiveCamera();
  private readonly frustum: Frustum = new Frustum();
  private readonly target: Vector3 = new Vector3();

  /**
   * @param changes - Where what the shadow views draw changed, and what sways or moves.
   */
  public constructor(changes: StaticShadowChanges) {
    this.changes = changes;
    this.changeVersion = changes.version;
    adoptRendererConventions(this.camera);
  }

  /** Forgets every light's faces, for lights put again. */
  public reset(): void {
    this.slots.clear();
    this.atlas.clear();
    this.queue.length = 0;
    this.askCount = 0;
    this.sizeScale = 1;
  }

  /** Keeps every face's square, but has each drawn again: the atlas lost what it held. */
  public forgetDrawn(): void {
    this.forEachEntry((entry: ILightShadowEntry) =>
      entry.faces.forEach((face: ILightShadowFace) => (face.isDrawn = false))
    );
  }

  /**
   * Brings the faces up to what changed since the last frame.
   *
   * @param isWindy - Whether the wind sways the trees this frame.
   */
  public begin(isWindy: boolean): void {
    const since: IShadowChanges = this.changes.since(this.changeVersion);

    this.frame += 1;
    this.isWindy = isWindy;
    this.askCount = 0;
    this.queue.length = 0;
    this.changeVersion = this.changes.version;

    if (since.isEverywhere) {
      this.forEachEntry((entry: ILightShadowEntry) => {
        entry.faces.forEach((face: ILightShadowFace) => (face.isStale = true));
        this.findMotion(entry);
      });
    } else {
      // Found once an entry, whatever reached it: a level arriving logs thousands of trees in one frame, and each
      // finding tests every caster that sways.
      since.changes.forEach((change: IShadowChange) => this.markChanged(change.box as Box3, change.isAnimated));
      this.moved.forEach((entry: ILightShadowEntry) => this.findMotion(entry));
      this.moved.clear();
    }
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
      const slot: ILightShadowSlot = this.getSlot(ask.index);
      const current: Nullable<ILightShadowEntry> = slot.next ?? slot.shown;
      const asked: number = toLightShadowTileSize(ask.engineSize * this.sizeScale, current?.asked ?? 0, isTight);
      const made: Nullable<ILightShadowEntry> = current?.asked === asked ? current : this.replace(ask, slot, asked);

      // Given less than it asked, or nothing: the asks want more than the atlas holds.
      if (!made || made.size < made.asked) {
        refused += 1;
      }

      (slot.next ?? slot.shown)?.faces.forEach((face: ILightShadowFace) => this.addCandidate(face, ask.distance));
    }

    this.fillQueue(budget);
    this.slots.forEach((slot: ILightShadowSlot) => this.promote(slot));
    this.fitScale(asks, refused);
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

  private getSlot(index: number): ILightShadowSlot {
    let slot: Maybe<ILightShadowSlot> = this.slots.get(index);

    if (!slot) {
      slot = { next: null, shown: null };
      this.slots.set(index, slot);
    }

    return slot;
  }

  /**
   * Makes a light's faces at a new size: drawn beside the shown ones while those are whole, so it never goes dark, and
   * in their place otherwise.
   *
   * @returns The faces made, or null where the atlas had no room.
   */
  private replace(ask: ILightShadowAsk, slot: ILightShadowSlot, asked: number): Nullable<ILightShadowEntry> {
    this.release(slot.next);
    slot.next = null;

    if (slot.shown && !this.isReady(slot.shown)) {
      this.release(slot.shown);
      slot.shown = null;
    }

    let entry: Nullable<ILightShadowEntry> = this.create(ask, asked);

    // No room beside the shown faces: the new ones take theirs, and the light waits for them.
    if (!entry && slot.shown) {
      this.release(slot.shown);
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
      this.release(slot.shown);
      slot.shown = slot.next;
      slot.next = null;
    }
  }

  private addCandidate(face: ILightShadowFace, distance: number): void {
    const isAnimated: boolean =
      face.motion === EShadowCasterMotion.MOVING || (face.motion === EShadowCasterMotion.SWAYING && this.isWindy);
    let urgency: EFaceUrgency;

    if (!face.isDrawn) {
      urgency = EFaceUrgency.UNDRAWN;
    } else if (face.isStale) {
      urgency = EFaceUrgency.STALE;
    } else if (isAnimated) {
      urgency = EFaceUrgency.ANIMATED;
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
    candidate.order = urgency === EFaceUrgency.ANIMATED ? face.drawnAt : distance;
  }

  private fillQueue(budget: number): void {
    const candidates: Array<IFaceCandidate> = takeSorted(
      this.candidates,
      this.candidateCount,
      this.orderedCandidates,
      byUrgency
    );

    for (let index: number = 0; index < Math.min(budget, candidates.length); index += 1) {
      this.queue.push(candidates[index].face);
    }
  }

  /**
   * Scales the asks down by what a refusal says, or back up by a step once the asks at the next scale fit, measured
   * exactly as the asks are rounded, over the lights in view alone.
   */
  private fitScale(asks: ReadonlyArray<ILightShadowAsk>, refused: number): void {
    const room: number = FILL * ATLAS_AREA;

    if (refused > 0) {
      const estimate: number = Math.sqrt(room / Math.max(toDemand(asks, 1), 1));

      this.sizeScale = Math.max(MIN_SCALE, Math.min(this.sizeScale * TIGHTEN, estimate));
    } else if (this.sizeScale < 1) {
      const next: number = Math.min(1, this.sizeScale * LOOSEN);

      if (toDemand(asks, next) <= room) {
        this.sizeScale = next;
      }
    }
  }

  /** Marks stale every face a change's box reaches, and looks again for what moves in the ones it could have moved. */
  private markChanged(box: Box3, isAnimated: boolean): void {
    this.forEachEntry((entry: ILightShadowEntry) => {
      if (!entry.sphere.intersectsBox(box)) {
        return;
      }

      entry.faces.forEach((face: ILightShadowFace) => {
        if (isBoxInPlanes(box, face.planes)) {
          face.isStale = true;
        }
      });

      if (isAnimated) {
        this.moved.add(entry);
      }
    });
  }

  private findMotion(entry: ILightShadowEntry): void {
    entry.faces.forEach((face: ILightShadowFace) => (face.motion = this.changes.getMotion(face.planes, entry.sphere)));
  }

  private forEachEntry(visit: (entry: ILightShadowEntry) => void): void {
    this.slots.forEach(({ shown, next }: ILightShadowSlot) => {
      if (shown) {
        visit(shown);
      }

      if (next) {
        visit(next);
      }
    });
  }

  /** A light's faces at the size asked, or the largest smaller that has room, or null for none. */
  private create(ask: ILightShadowAsk, asked: number): Nullable<ILightShadowEntry> {
    const count: number = toLightShadowFaceCount(ask.isSpot);

    for (let size: number = asked; size >= SMAP_MIN; size /= 2) {
      const tiles: Nullable<Array<ILightShadowTile>> = this.allocate(count, size);

      if (tiles) {
        const near: number = ask.near > 0 ? ask.near : DEFAULT_NEAR;
        const far: number = ask.range + FAR_EPSILON;
        const entry: ILightShadowEntry = {
          asked,
          faces: tiles.map((tile: ILightShadowTile, face: number) => this.createFace(ask, face, tile, near, far)),
          far,
          near,
          seen: this.frame,
          sphere: new Sphere(ask.position.clone(), ask.range),
          size,
        };

        this.findMotion(entry);

        return entry;
      }
    }

    return null;
  }

  /** Squares for every face, room made from the lights out of view longest where there is enough to make. */
  private allocate(count: number, size: number): Nullable<Array<ILightShadowTile>> {
    const needed: number = count * size * size;
    const evictable: Array<[number, ILightShadowSlot]> = [...this.slots].filter(([, slot]) => this.isEvictable(slot));

    if (ATLAS_AREA - this.atlas.used + evictable.reduce((total, [, slot]) => total + toSlotArea(slot), 0) < needed) {
      return null;
    }

    evictable.sort(([, a], [, b]) => toSlotSeen(a) - toSlotSeen(b));

    const tiles: Array<ILightShadowTile> = [];

    while (tiles.length < count) {
      const tile: Nullable<ILightShadowTile> = this.atlas.allocate(size);

      if (tile) {
        tiles.push(tile);
        continue;
      }

      const oldest: Maybe<[number, ILightShadowSlot]> = evictable.shift();

      if (!oldest) {
        tiles.forEach((it: ILightShadowTile) => this.atlas.release(it));

        return null;
      }

      this.release(oldest[1].shown);
      this.release(oldest[1].next);
      this.slots.delete(oldest[0]);
    }

    return tiles;
  }

  /** Whether a slot holds squares no light in view this frame is shown by. */
  private isEvictable({ shown, next }: ILightShadowSlot): boolean {
    return (shown !== null || next !== null) && (shown?.seen ?? -1) < this.frame && (next?.seen ?? -1) < this.frame;
  }

  private release(entry: Nullable<ILightShadowEntry>): void {
    entry?.faces.forEach((face: ILightShadowFace) => this.atlas.release(face.tile));
  }

  /** A face's matrices, as `compute_xf_spot` builds its camera: at the light, down its direction, the cone widened. */
  private createFace(
    ask: ILightShadowAsk,
    face: number,
    tile: ILightShadowTile,
    near: number,
    far: number
  ): ILightShadowFace {
    const { camera } = this;
    const cone: number = ask.isSpot ? ask.cone : LIGHT_SHADOW_POINT_CONE;
    const planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());

    camera.fov = ((cone + LIGHT_SHADOW_WIDENING) * 180) / Math.PI;
    camera.aspect = 1;
    camera.near = near;
    camera.far = far;
    camera.updateProjectionMatrix();
    camera.position.copy(ask.position);

    if (ask.isSpot) {
      camera.up.copy(ask.up);
      this.target.copy(ask.position).add(ask.direction);
    } else {
      const { direction, up } = LIGHT_SHADOW_POINT_FACES[face];

      camera.up.set(up[0], up[1], up[2]);
      this.target.set(direction[0], direction[1], direction[2]).add(ask.position);
    }

    camera.lookAt(this.target);
    camera.updateMatrixWorld();
    toPlaneVectors(toCameraFrustum(camera, this.frustum).planes, planes);

    return {
      drawnAt: 0,
      far,
      isDrawn: false,
      isStale: false,
      motion: EShadowCasterMotion.STILL,
      near,
      planes,
      projection: camera.projectionMatrix.clone(),
      tile,
      version: ++this.faceVersion,
      view: camera.matrixWorldInverse.clone(),
      world: camera.matrixWorld.clone(),
    };
  }
}

/** Texels the asks' faces take at a scale, each rounded as the scale rounds it. */
function toDemand(asks: ReadonlyArray<ILightShadowAsk>, scale: number): number {
  return asks.reduce((total: number, ask: ILightShadowAsk) => {
    const size: number = toLightShadowTileSize(ask.engineSize * scale, 0, scale < 1);

    return total + toLightShadowFaceCount(ask.isSpot) * size * size;
  }, 0);
}

function toSlotArea({ shown, next }: ILightShadowSlot): number {
  return [shown, next].reduce(
    (total: number, entry: Nullable<ILightShadowEntry>) =>
      total + (entry ? entry.faces.length * entry.size * entry.size : 0),
    0
  );
}

function toSlotSeen({ shown, next }: ILightShadowSlot): number {
  return Math.max(shown?.seen ?? -1, next?.seen ?? -1);
}

/**
 * @param pool - Objects kept between frames, the first `count` of them this frame's.
 * @param count - How many are this frame's.
 * @param out - Where they are sorted, kept between frames.
 * @param order - What they are sorted by.
 * @returns `out`, holding this frame's in order.
 */
function takeSorted<T>(pool: ReadonlyArray<T>, count: number, out: Array<T>, order: (a: T, b: T) => number): Array<T> {
  out.length = count;

  for (let index: number = 0; index < count; index += 1) {
    out[index] = pool[index];
  }

  return out.sort(order);
}

function byDistance(a: ILightShadowAsk, b: ILightShadowAsk): number {
  return a.distance - b.distance;
}

function byUrgency(a: IFaceCandidate, b: IFaceCandidate): number {
  return a.urgency - b.urgency || a.order - b.order;
}
