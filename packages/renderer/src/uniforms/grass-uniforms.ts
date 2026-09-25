import { uniform, uniformArray } from "three/tsl";
import { Frustum, Matrix4, PerspectiveCamera, Plane, Vector3, Vector4 } from "three/webgpu";

import { IRendererGrassSettings } from "#/contract/renderer-features";

/** Metres a detail slot spans, `dm_slot_size`. */
export const GRASS_SLOT_METERS: number = 2;

/** The least radius the engine plants to, `r__detail_radius`'s own floor. */
const LEAST_RADIUS: number = 49;

/**
 * What planting the grass around the camera reads: where the camera stands in the engine's space and on the slot grid,
 * how far the planting reaches, how densely it stands, and the view it is culled against.
 */
export class GrassUniforms {
  /** The eye, in the engine's space: renderer space with `z` negated. */
  public readonly eye = uniform(new Vector3());
  /** The slot the camera stands over, `iFloor(EYE / dm_slot_size + 0.5)` on each axis. */
  public readonly center = uniform(new Vector4());
  /** `dm_size`: slots planted each way from the camera's. */
  public readonly reach = uniform(0);
  /** `dm_fade`: metres from the eye at which a slot has shrunk to nothing. */
  public readonly fade = uniform(0);
  /** `d_size`: steps a slot's candidates are laid across, one more candidate than steps each way. */
  public readonly steps = uniform(0);
  /** How far a candidate is jittered off its step, `density / 1.7`. */
  public readonly jitter = uniform(0);
  /** What every tuft is scaled by, `ps_current_detail_height`. */
  public readonly height = uniform(1);
  /** The grid's size and where its first cell stands. */
  public readonly grid = uniform(new Vector4());
  /** The view's six planes in renderer space, pointing in. */
  public readonly planes: Array<Vector4> = Array.from({ length: 6 }, () => new Vector4());
  public readonly planeNodes = uniformArray(this.planes, "vec4");

  private readonly frustum: Frustum = new Frustum();
  private readonly viewProjection: Matrix4 = new Matrix4();

  /** Slots the planting covers, a square around the camera's: `dm_cache_line` squared. */
  public get slotCount(): number {
    const line: number = this.reach.value * 2 + 1;

    return line * line;
  }

  /** Candidates a slot lays out at most, `(d_size + 1)` squared. */
  public get candidateCount(): number {
    return (this.steps.value + 1) ** 2;
  }

  /**
   * @param settings - What the grass is set to.
   */
  public configure(settings: IRendererGrassSettings): void {
    const radius: number = Math.max(settings.radius, LEAST_RADIUS);
    const density: number = Math.min(Math.max(settings.density, 0.1), 0.99);

    // `dm_current_size` and `dm_current_fade`, from `r__detail_radius`.
    this.reach.value = Math.floor(radius / 4) * 2;
    this.fade.value = 2 * this.reach.value - 0.5;
    this.steps.value = Math.ceil(GRASS_SLOT_METERS / density);
    this.jitter.value = density / 1.7;
    this.height.value = settings.height;
  }

  /**
   * @param camera - The camera drawing, its matrices current.
   * @param sizeX - The grid's size in slots.
   * @param sizeZ - Its size the other way.
   * @param offsetX - How far its first cell stands from world slot zero.
   * @param offsetZ - The same the other way.
   */
  public follow(camera: PerspectiveCamera, sizeX: number, sizeZ: number, offsetX: number, offsetZ: number): void {
    const eye: Vector3 = camera.getWorldPosition(this.eye.value);

    eye.z = -eye.z;
    this.center.value.set(
      Math.floor(eye.x / GRASS_SLOT_METERS + 0.5),
      Math.floor(eye.z / GRASS_SLOT_METERS + 0.5),
      0,
      0
    );
    this.grid.value.set(sizeX, sizeZ, offsetX, offsetZ);
    this.viewProjection.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse);
    this.frustum.setFromProjectionMatrix(this.viewProjection, camera.coordinateSystem, camera.reversedDepth);
    this.frustum.planes.forEach((plane: Plane, index: number) =>
      this.planes[index].set(plane.normal.x, plane.normal.y, plane.normal.z, plane.constant)
    );
  }
}
