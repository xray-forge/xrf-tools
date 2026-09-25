import { Maybe, Nullable } from "@xrf/types";
import { Frustum, Matrix4, PerspectiveCamera, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { ILightShadowTile, LightShadowAtlas } from "#/scene/lights/light-shadow-atlas";
import {
  LIGHT_SHADOW_POINT_CONE,
  LIGHT_SHADOW_POINT_FACES,
  LIGHT_SHADOW_WIDENING,
} from "#/scene/lights/light-shadow-faces";

/** Texels the atlas is across. */
export const LIGHT_SHADOW_ATLAS_SIZE: number = 4096;

/** `SMAP_adapt_min`, `SMAP_adapt_optimal` and `SMAP_adapt_max` (`r2_types.h`). */
const SMAP_MIN: number = 32;
const SMAP_OPTIMAL: number = 768;
const SMAP_MAX: number = 1536;

/** The largest square a face takes, which leaves the atlas room for the rest. */
const TILE_MAX: number = 1024;

/** How far a face's wanted size may stray from its square before it is drawn at another: a square either way. */
const GROW: number = 1.5;
const SHRINK: number = 0.66;

/** `EPS_S`: what a face's far plane stands past the light's range. */
const FAR_EPSILON: number = 0.0000001;

/** Where a face's projection starts where the light gives none: `light::virtual_size`'s default. */
const DEFAULT_NEAR: number = 0.1;

/** A shadowed light as the planner takes it, in world space. */
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
  /** Metres from the camera to its sphere's edge, none inside it. */
  distance: number;
  /** `1 - dot(camera forward, light direction) / 2`: a light facing the camera earns more. */
  duel: number;
}

/** One face of a light's shadow: its camera, where it is drawn in the atlas, and when. */
export interface ILightShadowFace {
  readonly camera: PerspectiveCamera;
  /** Its frustum's planes, normals pointing in, what its casters are culled by. */
  readonly planes: ReadonlyArray<Vector4>;
  /** Its own, so the shadow view's last cull is never taken for another face's. */
  readonly version: number;
  readonly tile: ILightShadowTile;
  /** The casters' version it was drawn at, or null before it was drawn at all. */
  drawnVersion: Nullable<number>;
}

/** A shadowed light's faces, and the frame it was last in view. */
export interface ILightShadowEntry {
  readonly faces: ReadonlyArray<ILightShadowFace>;
  readonly size: number;
  readonly near: number;
  readonly far: number;
  seen: number;
}

/**
 * `CLight_Compute_XFORM_and_VIS::compute_xf_spot`'s map size: larger for a light nearer, brighter, facing the camera,
 * longer and wider, in texels.
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
 * @param current - The square it holds, or zero for none.
 * @returns The square it is to be drawn at: the power of two nearest the size, kept while the size stays within half a
 *   square of it, so a camera hovering between two does not draw it again every frame.
 */
export function toLightShadowTileSize(size: number, current: number): number {
  if (current > 0 && size <= current * GROW && size >= current * SHRINK) {
    return current;
  }

  const nearest: number = 2 ** Math.round(Math.log2(Math.max(size, 1)));

  return Math.min(Math.max(nearest, SMAP_MIN), TILE_MAX);
}

/**
 * Plans the lights' shadows: a square of the atlas a face, sized as the engine sizes its maps, and which faces are drawn
 * this frame. A face is drawn once and kept while nothing it casts from changed, a few faces a frame, the ones never
 * drawn first and the nearest first; a light whose faces are not all drawn lights unshadowed meanwhile. A light out of
 * view keeps its squares until another needs the room.
 */
export class LightShadows {
  public readonly atlas: LightShadowAtlas = new LightShadowAtlas(LIGHT_SHADOW_ATLAS_SIZE, SMAP_MIN);
  /** The faces to draw this frame, in order. */
  public queue: Array<ILightShadowFace> = [];

  private readonly entries: Map<number, ILightShadowEntry> = new Map();
  /** Faces wanting a draw this frame, with how urgently: lower first. */
  private candidates: Array<{ face: ILightShadowFace; priority: number }> = [];
  private frame: number = 0;
  private casterVersion: number = 0;
  private faceVersion: number = 0;
  private readonly frustum: Frustum = new Frustum();
  private readonly matrix: Matrix4 = new Matrix4();
  private readonly target: Vector3 = new Vector3();

  /** Forgets every light's faces, for lights put again or shadows turned off. */
  public reset(): void {
    this.entries.clear();
    this.atlas.clear();
    this.queue = [];
    this.candidates = [];
  }

  /**
   * @param casterVersion - What the casters are at: a face drawn at another is drawn again.
   */
  public begin(casterVersion: number): void {
    this.frame += 1;
    this.casterVersion = casterVersion;
    this.candidates = [];
    this.queue = [];
  }

  /**
   * @param index - The light, as the scene's lights number it.
   * @param request - Where it stands and how it wants its faces.
   * @returns Its faces, or null where the atlas has no room for them.
   */
  public request(index: number, request: ILightShadowRequest): Nullable<ILightShadowEntry> {
    let entry: Maybe<ILightShadowEntry> = this.entries.get(index);
    const size: number = toLightShadowTileSize(toLightShadowSize(request), entry?.size ?? 0);

    if (entry && entry.size !== size) {
      this.drop(index, entry);
      entry = undefined;
    }

    entry ??= this.create(index, request, size) ?? undefined;

    if (!entry) {
      return null;
    }

    entry.seen = this.frame;

    for (const face of entry.faces) {
      if (face.drawnVersion === null) {
        this.candidates.push({ face, priority: request.distance });
      } else if (face.drawnVersion !== this.casterVersion) {
        this.candidates.push({ face, priority: 1e9 + request.distance });
      }
    }

    return entry;
  }

  /**
   * Takes the faces drawn this frame.
   *
   * @param budget - Faces drawn at most.
   */
  public finish(budget: number): void {
    this.queue = this.candidates
      .sort((a, b) => a.priority - b.priority)
      .slice(0, budget)
      .map((candidate) => candidate.face);
  }

  /**
   * @param entry - A light's faces.
   * @returns Whether every face was drawn, or is drawn this frame, so the light can be shadowed by them.
   */
  public isReady(entry: ILightShadowEntry): boolean {
    return entry.faces.every((face: ILightShadowFace) => face.drawnVersion !== null || this.queue.includes(face));
  }

  /** Marks the queue drawn, at the casters' version. */
  public markDrawn(): void {
    for (const face of this.queue) {
      face.drawnVersion = this.casterVersion;
    }
  }

  /** A light's faces at a size, room made by the lights out of view longest where the atlas has none. */
  private create(index: number, request: ILightShadowRequest, size: number): Nullable<ILightShadowEntry> {
    const count: number = request.isSpot ? 1 : LIGHT_SHADOW_POINT_FACES.length;

    for (let wanted: number = size; wanted >= SMAP_MIN; wanted /= 2) {
      const tiles: Nullable<Array<ILightShadowTile>> = this.allocate(count, wanted);

      if (tiles) {
        const near: number = request.near > 0 ? request.near : DEFAULT_NEAR;
        const far: number = request.range + FAR_EPSILON;
        const entry: ILightShadowEntry = {
          faces: tiles.map((tile: ILightShadowTile, face: number) => this.createFace(request, face, tile, near, far)),
          far,
          near,
          seen: this.frame,
          size: wanted,
        };

        this.entries.set(index, entry);

        return entry;
      }
    }

    return null;
  }

  private allocate(count: number, size: number): Nullable<Array<ILightShadowTile>> {
    const tiles: Array<ILightShadowTile> = [];

    while (tiles.length < count) {
      const tile: Nullable<ILightShadowTile> = this.atlas.allocate(size);

      if (tile) {
        tiles.push(tile);
      } else if (!this.evict()) {
        tiles.forEach((it) => this.atlas.release(it));

        return null;
      }
    }

    return tiles;
  }

  /** Gives back the squares of the light out of view longest. */
  private evict(): boolean {
    let oldest: Nullable<number> = null;

    for (const [index, entry] of this.entries) {
      if (entry.seen < this.frame && (oldest === null || entry.seen < (this.entries.get(oldest)?.seen ?? 0))) {
        oldest = index;
      }
    }

    const entry: Maybe<ILightShadowEntry> = oldest === null ? undefined : this.entries.get(oldest);

    if (oldest === null || !entry) {
      return false;
    }

    this.drop(oldest, entry);

    return true;
  }

  private drop(index: number, entry: ILightShadowEntry): void {
    entry.faces.forEach((face: ILightShadowFace) => this.atlas.release(face.tile));
    this.entries.delete(index);
  }

  /** A face's camera, as `compute_xf_spot` builds it: at the light, down its direction, the cone widened a little. */
  private createFace(
    request: ILightShadowRequest,
    face: number,
    tile: ILightShadowTile,
    near: number,
    far: number
  ): ILightShadowFace {
    const cone: number = request.isSpot ? request.cone : LIGHT_SHADOW_POINT_CONE;
    const camera: PerspectiveCamera = new PerspectiveCamera(
      ((cone + LIGHT_SHADOW_WIDENING) * 180) / Math.PI,
      1,
      near,
      far
    );
    const planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());

    adoptRendererConventions(camera);
    camera.position.copy(request.position);

    if (request.isSpot) {
      camera.up.copy(request.up);
      this.target.copy(request.position).add(request.direction);
    } else {
      const basis = LIGHT_SHADOW_POINT_FACES[face];

      camera.up.set(basis.up[0], basis.up[1], basis.up[2]);
      this.target.set(basis.direction[0], basis.direction[1], basis.direction[2]).add(request.position);
    }

    camera.lookAt(this.target);
    camera.updateMatrixWorld();
    this.matrix.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse);
    this.frustum.setFromProjectionMatrix(this.matrix, camera.coordinateSystem, camera.reversedDepth);
    this.frustum.planes.forEach(({ normal, constant }, index: number) =>
      planes[index].set(normal.x, normal.y, normal.z, constant)
    );

    return { camera, drawnVersion: null, planes, tile, version: ++this.faceVersion };
  }
}
