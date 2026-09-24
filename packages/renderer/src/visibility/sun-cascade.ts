import { OrthographicCamera, PerspectiveCamera, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";

/** How far ahead of the camera a cascade's centre moves, in widths of the cascade, for a view across the light. */
const LEAD: number = 0.35;

/** How far a cascade's map reaches past its reach below its centre, in widths: the engine's `1.41421 * map_size`. */
const DEPTH: number = 1.41421;

/**
 * One cascade of the sun's shadow: a square of the level seen from the sun, fitted to the camera every frame. Its
 * centre leads the camera by a third of its width along where the camera looks, across the light, and is snapped to
 * the map's own texels, so moving the camera slides the map a whole texel at a time and its edges do not shimmer.
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
  private readonly key: Array<number> = [];

  public constructor() {
    adoptRendererConventions(this.camera);
  }

  /**
   * @param camera - The camera drawing, its world matrix current.
   * @param direction - Where the sun's light travels, normalized.
   * @param width - Metres the map is across.
   * @param resolution - Texels it is across.
   * @param reach - Metres towards the sun past the map that its casters may stand, and away from it its receivers.
   */
  public fit(camera: PerspectiveCamera, direction: Vector3, width: number, resolution: number, reach: number): void {
    const { right, up, light, center, look } = this;

    light.copy(direction).normalize();
    // The engine's basis: `x` across the light unless the light runs along it (`render_phase_sun.cpp`).
    right.set(1, 0, 0);

    if (Math.abs(right.dot(light)) > 0.99) {
      right.set(0, 0, 1);
    }

    up.crossVectors(light, right).normalize();
    right.crossVectors(up, light).normalize();

    // Ahead of the camera across the light, so the map covers what is looked at rather than what is behind.
    camera.getWorldDirection(look);
    look.addScaledVector(light, -look.dot(light));
    camera.getWorldPosition(center).addScaledVector(look, LEAD * width);

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

  private setPlane(index: number, normal: Vector3, constant: number): void {
    (this.planes[index] as Vector4).set(normal.x, normal.y, normal.z, constant);
  }
}
