import { Vector3 } from "three/webgpu";

/**
 * The axes a cascade's square is laid on, as the engine takes them (`render_phase_sun.cpp`): `x` across the light
 * unless the light runs along it.
 */
export class SunCascadeBasis {
  /** Across the light. */
  public readonly right: Vector3 = new Vector3();
  /** Across the light and `right`. */
  public readonly up: Vector3 = new Vector3();
  /** Where the light travels, normalized. */
  public readonly light: Vector3 = new Vector3();
  /** The square's four sides' normals, pointing in: along `right`, against it, along `up` and against it. */
  public readonly sides: ReadonlyArray<Vector3> = Array.from({ length: 4 }, () => new Vector3());

  /**
   * @param direction - Where the sun's light travels.
   */
  public take(direction: Vector3): void {
    const { right, up, light, sides } = this;

    light.copy(direction).normalize();
    right.set(1, 0, 0);

    if (Math.abs(right.dot(light)) > 0.99) {
      right.set(0, 0, 1);
    }

    up.crossVectors(light, right).normalize();
    right.crossVectors(up, light).normalize();
    sides[0].copy(right);
    sides[1].copy(right).negate();
    sides[2].copy(up);
    sides[3].copy(up).negate();
  }
}
