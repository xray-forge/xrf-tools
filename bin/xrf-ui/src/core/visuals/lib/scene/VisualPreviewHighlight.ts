import { Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, Object3D, Points, PointsMaterial } from "three";

import { IVisualPreviewSceneConfig } from "./scene-config";

/**
 * The marker for a joint named from outside the scene, such as a bone picked in a tree.
 */
export class VisualPreviewHighlight {
  /** After the model and the skeleton, since a marker drawn through both has to be drawn after them. */
  private static readonly RENDER_ORDER: number = 2;

  private readonly parent: Object3D;
  private readonly config: IVisualPreviewSceneConfig;

  private points: Nullable<Points<BufferGeometry, PointsMaterial>> = null;
  /** Where it points, kept so a toggle can show it again without the selection being sent a second time. */
  private position: Nullable<[number, number, number]> = null;
  private isRequested: boolean = false;

  public constructor(parent: Object3D, config: IVisualPreviewSceneConfig) {
    this.parent = parent;
    this.config = config;
  }

  /**
   * Points the marker at a place, or takes it off.
   *
   * @param position - Where to point, in the scene's own coordinates, or null to mark nothing.
   */
  public setPosition(position: Nullable<[number, number, number]>): void {
    this.position = position;

    if (position) {
      this.place(position);
    }

    this.applyVisibility();
  }

  /**
   * @param isVisible - Whether the overlay the marker belongs to is switched on at all.
   */
  public setRequested(isVisible: boolean): void {
    this.isRequested = isVisible;

    this.applyVisibility();
  }

  /** Releases the marker and takes it out of the scene. */
  public dispose(): void {
    if (this.points) {
      this.parent.remove(this.points);
      this.points.geometry.dispose();
      this.points.material.dispose();
      this.points = null;
    }

    this.position = null;
  }

  private place(position: [number, number, number]): void {
    const points: Points<BufferGeometry, PointsMaterial> = this.points ?? this.create();
    const attribute: BufferAttribute = points.geometry.getAttribute("position") as BufferAttribute;

    attribute.setXYZ(0, position[0], position[1], position[2]);
    attribute.needsUpdate = true;

    // A single point, so its bounding sphere is stale after a move and frustum culling would drop it.
    points.geometry.computeBoundingSphere();
  }

  private create(): Points<BufferGeometry, PointsMaterial> {
    const geometry: BufferGeometry = new BufferGeometry();

    geometry.setAttribute("position", new BufferAttribute(new Float32Array(3), 3));

    const points: Points<BufferGeometry, PointsMaterial> = new Points(
      geometry,
      new PointsMaterial({
        color: this.config.highlightColor,
        depthTest: false,
        size: this.config.highlightSize,
        sizeAttenuation: false,
        transparent: true,
      })
    );

    points.renderOrder = VisualPreviewHighlight.RENDER_ORDER;

    this.points = points;
    this.parent.add(points);

    return points;
  }

  private applyVisibility(): void {
    if (this.points) {
      this.points.visible = Boolean(this.position) && this.isRequested;
    }
  }
}
