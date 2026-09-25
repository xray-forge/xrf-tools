import { OrthographicCamera, PerspectiveCamera, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { ISunViewRay, SunViewRays } from "#/visibility/sun-view-rays";

/** How far a cascade's map reaches past its reach below its centre, in widths: the engine's `1.41421 * map_size`. */
const DEPTH: number = 1.41421;

/** The engine's `EPS_L`: a plane faces the view only past it, and a ray leaves a plane only past it. */
const EPS_L: number = 0.001;

/** The engine's `EPS_S`, under which a view along the light places nothing. */
const EPS_S: number = 0.0000001;

/** The engine's first guess at the nearest point behind a side, which any real one is nearer than. */
const FAR_BEHIND: number = 10000;

/**
 * How deep from the near plane, in widths, a cascade holds the whole view whatever the engine's placement says.
 * Brought up to where the edges start, a square leaves out the ground an edge runs back out through its back side to,
 * which is the ground nearest the camera once it looks down with the sun ahead; the engine's far pass reads that from
 * its map's clamped edge. A quarter of a width fits in the square at any angle.
 */
const HELD: number = 0.25;

/** The share of the width a held point is kept inside the square by: the sampling's own edge, and a little over. */
const HELD_MARGIN: number = 0.03;

/**
 * One cascade of the sun's shadow: a square of the level seen from the sun, placed every frame as the engine places it
 * (`compute_caster_model_fixed`). The sides of the square the view looks away from are brought up to the nearest
 * point the view's edges start from, so the square covers what is ahead rather than what is around; then the edges
 * are carried to where they leave it, for the next cascade to start there. The centre is snapped to the map's own
 * texels, so moving the camera slides the map a whole texel at a time and its edges do not shimmer.
 */
export class SunCascade {
  /** What the cascade is drawn from: looking along the light, the square's width across. */
  public readonly camera: OrthographicCamera = new OrthographicCamera();
  /** Its box's six planes, normals pointing in, `w` the constant: what its casters are culled by. */
  public readonly planes: ReadonlyArray<Vector4> = Array.from({ length: 6 }, () => new Vector4());
  /** Metres one texel of its map is across. */
  public texel: number = 0;
  /** Metres the map is across. */
  public width: number = 0;
  /** Bumped whenever the map moved, so its casters are culled again. */
  public version: number = 0;

  private readonly right: Vector3 = new Vector3();
  private readonly up: Vector3 = new Vector3();
  private readonly light: Vector3 = new Vector3();
  private readonly center: Vector3 = new Vector3();
  private readonly look: Vector3 = new Vector3();
  /** The square's four sides' normals, pointing in: along `right`, against it, along `up` and against it. */
  private readonly sides: ReadonlyArray<Vector3> = Array.from({ length: 4 }, () => new Vector3());
  private readonly translation: Vector3 = new Vector3();
  private readonly push: Vector3 = new Vector3();
  private readonly across: Vector3 = new Vector3();
  private readonly key: Array<number> = [];

  public constructor() {
    adoptRendererConventions(this.camera);
  }

  /**
   * @param camera - The camera drawing, its world matrix current.
   * @param rays - The view's edges where this cascade starts, carried on to where the next one starts.
   * @param direction - Where the sun's light travels, normalized.
   * @param width - Metres the map is across.
   * @param resolution - Texels it is across.
   * @param reach - Metres towards the sun past the map that its casters may stand, and away from it its receivers.
   */
  public fit(
    camera: PerspectiveCamera,
    rays: SunViewRays,
    direction: Vector3,
    width: number,
    resolution: number,
    reach: number
  ): void {
    const { right, up, light, center, look } = this;

    light.copy(direction).normalize();
    // The engine's basis: `x` across the light unless the light runs along it (`render_phase_sun.cpp`).
    right.set(1, 0, 0);

    if (Math.abs(right.dot(light)) > 0.99) {
      right.set(0, 0, 1);
    }

    up.crossVectors(light, right).normalize();
    right.crossVectors(up, light).normalize();

    this.sides[0].copy(right);
    this.sides[1].copy(right).negate();
    this.sides[2].copy(up);
    this.sides[3].copy(up).negate();

    camera.getWorldPosition(center);
    camera.getWorldDirection(look);

    // The engine's light stands over the camera: the square starts centred on it, and is then moved across the light.
    this.place(center, look, rays, width);

    const texel: number = width / resolution;
    const x: number = Math.round(center.dot(right) / texel) * texel;
    const y: number = Math.round(center.dot(up) / texel) * texel;
    const z: number = center.dot(light);
    const half: number = width / 2;
    // As far below the centre as towards the sun, and the engine's margin past it: a camera flying high still has its
    // ground in the map.
    const far: number = 2 * reach + DEPTH * width;

    center.set(0, 0, 0).addScaledVector(right, x).addScaledVector(up, y).addScaledVector(light, z);

    const key: Array<number> = [x, y, z, width, resolution, reach, light.x, light.y, light.z];

    if (key.some((value: number, index: number) => value !== this.key[index])) {
      this.key.splice(0, key.length, ...key);
      this.version += 1;
    }

    this.texel = texel;
    this.width = width;
    this.camera.up.copy(up);
    this.camera.position.copy(center).addScaledVector(light, -reach);
    this.camera.lookAt(look.copy(center));
    this.camera.left = -half;
    this.camera.right = half;
    this.camera.top = half;
    this.camera.bottom = -half;
    this.camera.near = 0;
    this.camera.far = far;
    this.camera.updateProjectionMatrix();
    this.camera.updateMatrixWorld(true);

    // The same box as planes: across the light either way, and from the reach towards the sun to its far side.
    this.setPlane(0, right, -(x - half));
    this.setPlane(1, right.clone().negate(), x + half);
    this.setPlane(2, up, -(y - half));
    this.setPlane(3, up.clone().negate(), y + half);
    this.setPlane(4, light, -(z - reach));
    this.setPlane(5, light.clone().negate(), z - reach + far);
  }

  /**
   * Moves the square, centred on `center`, across the light to cover the view ahead, holds the first stretch of the
   * view in it, and carries the edges to where they leave it: `compute_caster_model_fixed`, step by step, and `HELD`.
   *
   * @param center - The square's centre, moved in place.
   * @param look - Where the camera looks.
   * @param rays - The view's edges, carried on in place.
   * @param width - Metres the square is across.
   */
  private place(center: Vector3, look: Vector3, rays: SunViewRays, width: number): void {
    // Looking along the light, no side faces away from the view and the edges stay where they are.
    if (Math.abs(1 - Math.abs(look.dot(this.light))) < EPS_S) {
      this.hold(center, look, rays.near, width);

      return;
    }

    this.align(center, look, rays.rays, width);
    this.hold(center, look, rays.near, width);
    this.advance(center, rays.rays, width);
  }

  /**
   * Brings the sides the view looks away from up to the nearest point an edge starts from, then back by the share an
   * edge running out through them leaves at.
   *
   * @param center - The square's centre, moved in place.
   * @param look - Where the camera looks.
   * @param rays - Where the view's edges start for this cascade.
   * @param width - Metres the square is across.
   */
  private align(center: Vector3, look: Vector3, rays: ReadonlyArray<ISunViewRay>, width: number): void {
    const { sides, translation, push, across } = this;
    const half: number = width / 2;

    // The one or two sides the view looks away from, behind the camera.
    const behind: Array<number> = [0, 1, 2, 3].filter((side: number) => look.dot(sides[side]) > EPS_L).slice(0, 2);

    // Each brought up to the nearest point an edge starts from.
    translation.set(0, 0, 0);

    for (const side of behind) {
      const nearest: number = rays.reduce(
        (least: number, ray: ISunViewRay) => Math.min(least, this.inside(side, ray.origin, center, half)),
        FAR_BEHIND
      );

      translation.addScaledVector(sides[side], nearest);
    }

    // An edge running back out through a side it was brought up to pulls that side back by the share it leaves at.
    push.set(0, 0, 0);

    for (const side of behind) {
      const normal: Vector3 = sides[side];
      let share: number = 0;

      across.crossVectors(normal, look).cross(look);

      for (const ray of rays) {
        const along: number = ray.direction.dot(normal);

        if (along < 0) {
          share = Math.max(share, -along / ray.direction.dot(across));
        }
      }

      if (Math.abs(share) >= EPS_S) {
        push.addScaledVector(normal, -normal.dot(translation) * share);
      }
    }

    center.add(translation).add(push);
  }

  /**
   * Moves the square across the light no further than it must to hold the view from the near plane to `HELD` of its
   * width deep, which fits in it at any angle: the slice's widest extent is under three quarters of a width.
   *
   * @param center - The square's centre, moved in place.
   * @param look - Where the camera looks, which the depth is measured along.
   * @param near - The view's edges from the near plane.
   * @param width - Metres the square is across.
   */
  private hold(center: Vector3, look: Vector3, near: ReadonlyArray<ISunViewRay>, width: number): void {
    const reach: number = width * HELD;
    const half: number = width * (0.5 - HELD_MARGIN);

    for (const axis of [this.right, this.up]) {
      let least: number = Infinity;
      let most: number = -Infinity;

      for (const ray of near) {
        const start: number = axis.dot(ray.origin);
        const end: number = start + (axis.dot(ray.direction) * reach) / ray.direction.dot(look);

        least = Math.min(least, start, end);
        most = Math.max(most, start, end);
      }

      const at: number = axis.dot(center);
      const lowest: number = most - half;
      const highest: number = least + half;
      const held: number = lowest > highest ? (least + most) / 2 : Math.min(Math.max(at, lowest), highest);

      center.addScaledVector(axis, held - at);
    }
  }

  /**
   * Carries each edge to where it leaves the square, which is where the next cascade starts it.
   *
   * @param center - The square's centre.
   * @param rays - The view's edges, carried on in place.
   * @param width - Metres the square is across.
   */
  private advance(center: Vector3, rays: ReadonlyArray<ISunViewRay>, width: number): void {
    const { sides } = this;
    const half: number = width / 2;

    for (const ray of rays) {
      let nearest: number = 2 * width;

      for (let side = 0; side < sides.length; side += 1) {
        const along: number = sides[side].dot(ray.direction);
        let distance: number = width;

        if (along <= -0.1) {
          const leave: number = -this.inside(side, ray.origin, center, half) / along;

          distance = leave > 0 || Math.abs(leave) < EPS_S ? leave : 0;
        }

        if (distance > EPS_L && distance < nearest) {
          nearest = distance;
        }
      }

      ray.origin.addScaledVector(ray.direction, nearest);
    }
  }

  /**
   * @param side - One of the square's sides.
   * @param point - A point.
   * @param center - The square's centre.
   * @param half - Half its width.
   * @returns How far inside that side the point stands: the engine's `classify` against it.
   */
  private inside(side: number, point: Vector3, center: Vector3, half: number): number {
    const normal: Vector3 = this.sides[side];

    return normal.dot(point) - normal.dot(center) + half;
  }

  private setPlane(index: number, normal: Vector3, constant: number): void {
    (this.planes[index] as Vector4).set(normal.x, normal.y, normal.z, constant);
  }
}
