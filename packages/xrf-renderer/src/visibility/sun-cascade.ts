import { OrthographicCamera, PerspectiveCamera, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { SunCascadeBasis } from "#/visibility/sun-cascade-basis";
import { placeSunCascade } from "#/visibility/sun-cascade-placement";
import { SunViewRays } from "#/visibility/sun-view-rays";

/** Numbers a cascade is fitted by: its square's place, width, resolution, reach, and the light's direction. */
const KEY_LENGTH: number = 9;

/** How far a cascade's map reaches past its reach below its centre, in widths: the engine's `1.41421 * map_size`. */
const DEPTH: number = 1.41421;

/** The step, in widths, a cascade's place along the light is rounded to, so a move along it alone keeps the map. */
const DEPTH_STEP: number = 1 / 16;

/**
 * One cascade of the sun's shadow: a square of the level seen from the sun, placed every frame as the engine places it
 * (`compute_caster_model_fixed`), then snapped to the map's own texels, so moving the camera slides the map a whole
 * texel at a time and its edges do not shimmer.
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

  private readonly basis: SunCascadeBasis = new SunCascadeBasis();
  private readonly center: Vector3 = new Vector3();
  private readonly look: Vector3 = new Vector3();
  /** What the cascade was last fitted by: its square's place, width, resolution, reach and the light. */
  private readonly key: Float64Array = new Float64Array(KEY_LENGTH).fill(NaN);
  /** What it is fitted by this frame, compared with the key before the key takes it. */
  private readonly next: Float64Array = new Float64Array(KEY_LENGTH);
  private readonly negated: Vector3 = new Vector3();

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
    const { basis, center, look } = this;
    const { right, up, light } = basis;

    basis.take(direction);
    camera.getWorldPosition(center);
    camera.getWorldDirection(look);

    // The engine's light stands over the camera: the square starts centred on it, and is then moved across the light.
    placeSunCascade(center, look, rays, basis, width);

    const texel: number = width / resolution;
    const x: number = Math.round(center.dot(right) / texel) * texel;
    const y: number = Math.round(center.dot(up) / texel) * texel;
    const step: number = width * DEPTH_STEP;
    const z: number = Math.round(center.dot(light) / step) * step;
    const half: number = width / 2;
    // A step further each way than the rounding strays, towards the sun as far as the reach and below the centre as far
    // again and the engine's margin: a camera flying high still has its ground in the map.
    const back: number = reach + step;
    const far: number = 2 * back + DEPTH * width;

    center.set(0, 0, 0).addScaledVector(right, x).addScaledVector(up, y).addScaledVector(light, z);

    if (this.rekey(x, y, z, width, resolution, reach)) {
      this.version += 1;
    }

    this.texel = texel;
    this.width = width;
    this.camera.up.copy(up);
    this.camera.position.copy(center).addScaledVector(light, -back);
    this.camera.lookAt(look.copy(center));
    this.camera.left = -half;
    this.camera.right = half;
    this.camera.top = half;
    this.camera.bottom = -half;
    this.camera.near = 0;
    this.camera.far = far;
    this.camera.updateProjectionMatrix();
    this.camera.updateMatrixWorld(true);

    // The same box as planes: across the light either way, and from its near side towards the sun to its far side.
    this.setPlane(0, right, -(x - half));
    this.setPlane(1, this.negated.copy(right).negate(), x + half);
    this.setPlane(2, up, -(y - half));
    this.setPlane(3, this.negated.copy(up).negate(), y + half);
    this.setPlane(4, light, -(z - back));
    this.setPlane(5, this.negated.copy(light).negate(), z - back + far);
  }

  /**
   * Takes this frame's fit as the key where it differs from the last.
   *
   * @returns Whether it differed, which is a new version of the cascade.
   */
  private rekey(x: number, y: number, z: number, width: number, resolution: number, reach: number): boolean {
    const { key, next } = this;
    const { light } = this.basis;

    next[0] = x;
    next[1] = y;
    next[2] = z;
    next[3] = width;
    next[4] = resolution;
    next[5] = reach;
    next[6] = light.x;
    next[7] = light.y;
    next[8] = light.z;

    for (let index: number = 0; index < KEY_LENGTH; index += 1) {
      if (next[index] !== key[index]) {
        key.set(next);

        return true;
      }
    }

    return false;
  }

  private setPlane(index: number, normal: Vector3, constant: number): void {
    this.planes[index].set(normal.x, normal.y, normal.z, constant);
  }
}
