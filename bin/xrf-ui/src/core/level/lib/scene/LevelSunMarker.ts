import { Color, Mesh, MeshBasicMaterial, Object3D, SphereGeometry, Vector3 } from "three";

import { markThrough } from "@/core/render/lib/scene/render-marker";

import { SUN_MARKER_DISTANCE, SUN_MARKER_RADIUS, SUN_MARKER_SEGMENTS } from "./level-lighting-config";

/**
 * Where the light is coming from, drawn as a disc in the sky.
 */
export class LevelSunMarker {
  private readonly parent: Object3D;
  private readonly material: MeshBasicMaterial = new MeshBasicMaterial();
  private readonly mesh: Mesh<SphereGeometry, MeshBasicMaterial>;

  /** Unit vector towards the sun, which is the marker's bearing from wherever the camera stands. */
  private readonly direction: Vector3 = new Vector3(0, 1, 0);
  private readonly origin: Vector3 = new Vector3();

  public constructor(parent: Object3D) {
    this.parent = parent;
    this.mesh = new Mesh(
      new SphereGeometry(SUN_MARKER_RADIUS, SUN_MARKER_SEGMENTS, SUN_MARKER_SEGMENTS),
      this.material
    );

    markThrough(this.mesh);

    this.parent.add(this.mesh);
    this.place();
  }

  /**
   * Points the marker at where the light is coming from.
   *
   * @param direction - Where the sun sits relative to what it lights, of any length.
   * @param color - Hex colour of the light, which is what the marker is drawn in.
   */
  public setSun(direction: Vector3, color: number): void {
    // Guarded because a direction of no length has no bearing to take, and normalising it would leave the marker at
    // the camera's own eye.
    if (direction.lengthSq() > 0) {
      this.direction.copy(direction).normalize();
    }

    this.material.color = new Color(color);

    this.place();
  }

  /**
   * Keeps the marker in the sky over wherever the camera has flown to.
   *
   * @param position - Where the camera is now.
   */
  public follow(position: Vector3): void {
    if (this.origin.equals(position)) {
      return;
    }

    this.origin.copy(position);

    this.place();
  }

  /**
   * @param isVisible - Whether the marker is drawn at all.
   */
  public setVisible(isVisible: boolean): void {
    this.mesh.visible = isVisible;
  }

  /** Releases the marker's own geometry and material and takes it out of the scene. */
  public dispose(): void {
    this.parent.remove(this.mesh);

    this.mesh.geometry.dispose();
    this.material.dispose();
  }

  private place(): void {
    this.mesh.position.copy(this.origin).addScaledVector(this.direction, SUN_MARKER_DISTANCE);
  }
}
