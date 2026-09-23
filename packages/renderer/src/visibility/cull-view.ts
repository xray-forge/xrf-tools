import { Frustum, Matrix4, PerspectiveCamera, Plane, Sphere } from "three/webgpu";

import { EVisibility } from "#/visibility/visibility";

/**
 * What one view sees: its camera's frustum, the far plane being as far as it sees.
 * A view keeps its version while its camera stays put, so whatever was culled against it stays culled.
 */
export class CullView {
  private readonly frustum: Frustum = new Frustum();
  private readonly projection: Matrix4 = new Matrix4();
  private readonly next: Matrix4 = new Matrix4();
  private currentVersion: number = 0;

  /** The frustum's six planes, normals pointing in. */
  public get planes(): ReadonlyArray<Plane> {
    return this.frustum.planes;
  }

  /** Bumped whenever the view sees something else. */
  public get version(): number {
    return this.currentVersion;
  }

  /**
   * @param camera - The camera the view is taken from, with its matrices current.
   */
  public take(camera: PerspectiveCamera): void {
    this.next.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse);

    if (this.currentVersion && this.next.equals(this.projection)) {
      return;
    }

    this.projection.copy(this.next);
    this.frustum.setFromProjectionMatrix(this.projection, camera.coordinateSystem);
    this.currentVersion += 1;
  }

  /**
   * @param x - The sphere's centre, in renderer space.
   * @param y - Its centre.
   * @param z - Its centre.
   * @param radius - Its radius.
   * @returns How much of the sphere the view sees.
   */
  public classify(x: number, y: number, z: number, radius: number): EVisibility {
    let isInside: boolean = true;

    for (const { normal, constant } of this.frustum.planes as ReadonlyArray<Plane>) {
      const distance: number = normal.x * x + normal.y * y + normal.z * z + constant;

      if (distance < -radius) {
        return EVisibility.OUTSIDE;
      }

      if (distance < radius) {
        isInside = false;
      }
    }

    return isInside ? EVisibility.INSIDE : EVisibility.INTERSECTS;
  }

  /**
   * @param sphere - A sphere in renderer space.
   * @returns How much of it the view sees.
   */
  public classifySphere(sphere: Sphere): EVisibility {
    return this.classify(sphere.center.x, sphere.center.y, sphere.center.z, sphere.radius);
  }
}
