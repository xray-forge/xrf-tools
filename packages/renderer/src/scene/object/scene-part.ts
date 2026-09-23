import { Maybe, Nullable } from "@xrf/types";
import {
  BufferGeometry,
  InstancedBufferGeometry,
  Material,
  Matrix4,
  Mesh,
  Scene,
  Skeleton,
  Sphere,
} from "three/webgpu";

import { IRendererObject } from "#/contract/scene/renderer-object";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { createSceneMesh } from "#/scene/object/scene-mesh";

/**
 * One section of an object as the frame draws it: a mesh over the section's range, in the scene of the pass its
 * surface is drawn by, bounded where it stands.
 */
export class ScenePart {
  /** Its own geometry over the object's buffers, which it disposes. */
  public readonly geometry: BufferGeometry;
  /** Its section's position among the geometry's, which its surface is looked up by. */
  public readonly section: number;
  /** What it spans in renderer space. */
  public readonly sphere: Sphere = new Sphere();

  private readonly source: ISceneSection;
  private currentMesh: Mesh;
  /** Whether the object's narrowing leaves it anything to draw. */
  private isDrawable: boolean = true;

  /**
   * @param geometry - Its part geometry.
   * @param section - Its section's position among the geometry's.
   * @param source - The section itself.
   * @param skeleton - What its mesh is skinned to, or null.
   */
  public constructor(geometry: BufferGeometry, section: number, source: ISceneSection, skeleton: Nullable<Skeleton>) {
    this.geometry = geometry;
    this.section = section;
    this.source = source;
    this.currentMesh = createSceneMesh(geometry, skeleton);
  }

  public get mesh(): Mesh {
    return this.currentMesh;
  }

  /**
   * @param skeleton - What its mesh is skinned to now; a new mesh over the same geometry, shown nowhere yet.
   */
  public remesh(skeleton: Nullable<Skeleton>): void {
    this.currentMesh.removeFromParent();
    this.currentMesh = createSceneMesh(this.geometry, skeleton);
  }

  /**
   * @param range - The index range the object narrows its geometry to, if any.
   */
  public narrow(range: Maybe<NonNullable<IRendererObject["drawRange"]>>): void {
    const start: number = Math.max(this.source.start, range?.start ?? 0);
    const end: number = Math.min(this.source.start + this.source.count, range ? range.start + range.count : Infinity);

    this.isDrawable = end > start;
    this.geometry.setDrawRange(start, Math.max(0, end - start));
  }

  /**
   * @param matrix - Where the object stands.
   */
  public place(matrix: Matrix4): void {
    this.currentMesh.matrix.copy(matrix);
    this.currentMesh.updateMatrixWorld(true);
    this.sphere.copy(this.source.sphere).applyMatrix4(matrix);
  }

  /**
   * @param material - What draws it, or null for a section whose surface is missing.
   * @param scene - The scene of the pass drawing that material.
   */
  public show(material: Nullable<Material>, scene: Nullable<Scene>): void {
    if (material && scene) {
      this.currentMesh.material = material;
      scene.add(this.currentMesh);
    } else {
      this.currentMesh.removeFromParent();
    }
  }

  /**
   * @param isSeen - Whether the view drawn for sees it.
   */
  public cull(isSeen: boolean): void {
    this.currentMesh.visible = this.isDrawable && isSeen;
  }

  /**
   * @param count - How many of its instances the view sees.
   */
  public cullInstances(count: number): void {
    (this.geometry as InstancedBufferGeometry).instanceCount = count;
    this.cull(count > 0);
  }

  public detach(): void {
    this.currentMesh.removeFromParent();
  }
}
