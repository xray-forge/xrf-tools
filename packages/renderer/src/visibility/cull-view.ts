import { Frustum, Matrix4, PerspectiveCamera, Plane, Sphere, Vector3 } from "three/webgpu";

import { EVisibility } from "#/visibility/visibility";

/** Where the far plane of three's frustum sits among its planes. */
const FAR_PLANE: number = 4;

/**
 * What one view sees: its camera's frustum, its far plane brought in to how far the view sees. The camera keeps its
 * own far plane, so helpers drawn past the view distance share the scene's depth.
 * A view keeps its version while its camera and distance stay put, so whatever was culled against it stays culled.
 */
export class CullView {
  private readonly frustum: Frustum = new Frustum();
  private readonly projection: Matrix4 = new Matrix4();
  private readonly next: Matrix4 = new Matrix4();
  private readonly forward: Vector3 = new Vector3();
  private readonly point: Vector3 = new Vector3();
  private distance: number = Infinity;
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
   * @param distance - How far the view sees, where that is nearer than the camera's far plane.
   */
  public take(camera: PerspectiveCamera, distance: number = Infinity): void {
    this.next.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse);

    if (this.currentVersion && this.distance === distance && this.next.equals(this.projection)) {
      return;
    }

    this.projection.copy(this.next);
    this.distance = distance;
    this.frustum.setFromProjectionMatrix(this.projection, camera.coordinateSystem, camera.reversedDepth);

    if (distance < camera.far) {
      // Facing the camera, through the point the distance ahead of it.
      camera.getWorldDirection(this.forward);
      camera.getWorldPosition(this.point).addScaledVector(this.forward, distance);
      this.frustum.planes[FAR_PLANE].setFromNormalAndCoplanarPoint(this.forward.negate(), this.point);
    }

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
