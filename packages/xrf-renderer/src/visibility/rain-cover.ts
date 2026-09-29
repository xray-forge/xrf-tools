import { OrthographicCamera, Vector3, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";

/**
 * Metres the cover is across: every streak's column falls within it, however far the camera is from its centre, and
 * every surface the rain wets, 25 metres out at most on the extended engine.
 */
export const RAIN_COVER_WIDTH: number = 56;

/** Texels it is across, some five centimetres each. */
export const RAIN_COVER_RESOLUTION: number = 1024;

/** Metres above the camera it is seen from, past where every streak starts. */
const HEIGHT: number = 60;

/** Metres it reaches down from there. */
export const RAIN_COVER_DEPTH: number = 120;

/** Metres the cover moves by, so a camera moving less draws it again for nothing. */
const STEP: number = 4;

/**
 * What stands over the rain around the camera, seen straight down: a square of the level whose depth says, for each
 * column, how high the first thing a drop lands on is. It moves a whole step at a time.
 */
export class RainCover {
  /** What it is drawn from: looking down, the square's width across, `+x` right and `-z` up. */
  public readonly camera: OrthographicCamera = new OrthographicCamera();
  /** Its box's six planes, normals pointing in, `w` the constant: what its casters are culled by. */
  public readonly planes: ReadonlyArray<Vector4> = Array.from({ length: 6 }, () => new Vector4());
  /** Where it is centred, and the height it is seen from. */
  public readonly center: Vector3 = new Vector3(NaN, NaN, NaN);
  /** Bumped whenever it moved, so its casters are culled again. */
  public version: number = 0;

  public constructor() {
    const half: number = RAIN_COVER_WIDTH / 2;

    adoptRendererConventions(this.camera);
    this.camera.up.set(0, 0, -1);
    this.camera.left = -half;
    this.camera.right = half;
    this.camera.top = half;
    this.camera.bottom = -half;
    this.camera.near = 0;
    this.camera.far = RAIN_COVER_DEPTH;
    this.camera.updateProjectionMatrix();
  }

  /**
   * @param position - Where the camera stands, in renderer space.
   * @returns Whether the cover moved.
   */
  public fit(position: Vector3): boolean {
    const x: number = Math.round(position.x / STEP) * STEP;
    const z: number = Math.round(position.z / STEP) * STEP;
    const top: number = Math.round(position.y / STEP) * STEP + HEIGHT;

    if (x === this.center.x && z === this.center.z && top === this.center.y) {
      return false;
    }

    const half: number = RAIN_COVER_WIDTH / 2;

    this.center.set(x, top, z);
    this.camera.position.set(x, top, z);
    this.camera.lookAt(x, top - 1, z);
    this.camera.updateMatrixWorld(true);
    this.planes[0].set(1, 0, 0, -(x - half));
    this.planes[1].set(-1, 0, 0, x + half);
    this.planes[2].set(0, 0, 1, -(z - half));
    this.planes[3].set(0, 0, -1, z + half);
    this.planes[4].set(0, 1, 0, -(top - RAIN_COVER_DEPTH));
    this.planes[5].set(0, -1, 0, top);
    this.version += 1;

    return true;
  }
}
