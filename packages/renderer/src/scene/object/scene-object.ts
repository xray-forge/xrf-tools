import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Material, Matrix4, Mesh, Scene, Skeleton, SkinnedMesh } from "three/webgpu";

import { IRendererInstances, IRendererObject } from "#/contract/scene/renderer-object";
import { SceneInstances } from "#/scene/object/scene-instances";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { RENDERER_PASSES, toPassRecord, TPassRecord } from "#/scene/pass-record";
import { HIDDEN_MATERIAL } from "#/scene/surface/hidden-material";

/**
 * One object a consumer put: a mesh per pass over one geometry, each drawing only its own slots.
 */
export class SceneObject {
  /**
   * A mesh the object's matrix places directly, never recomposed from a position and rotation.
   *
   * @param skeleton - What it is skinned to, or null for a rigid mesh.
   * @returns The mesh.
   */
  public static createMesh(skeleton: Nullable<Skeleton>): Mesh {
    const mesh: Mesh = skeleton ? new SkinnedMesh() : new Mesh();

    mesh.matrixAutoUpdate = false;

    if (mesh instanceof SkinnedMesh && skeleton) {
      // Measured against its bind pose, a skinned mesh would be culled by a motion reaching outside it.
      mesh.frustumCulled = false;
      // Identity: the vertices and the bone transforms are both in model space already.
      mesh.bind(skeleton, new Matrix4());
    }

    return mesh;
  }

  /** The key it was put under, which a released object no longer answers to. */
  public readonly key: string;
  public object: IRendererObject;

  private meshes: TPassRecord<Mesh> = toPassRecord(() => SceneObject.createMesh(null));
  /** The skeleton its meshes are bound to, or null for rigid meshes. */
  private skeleton: Nullable<Skeleton> = null;
  /** The places its meshes stand, or null for a mesh its matrix places once. */
  private instances: Nullable<SceneInstances> = null;
  /** The places it is about to stand in, built for the change it waits in. */
  private staged: Nullable<SceneInstances> = null;

  public constructor(key: string, object: IRendererObject) {
    this.key = key;
    this.object = object;
  }

  /** Every mesh drawing it, whether or not a scene holds it. */
  public get drawing(): ReadonlyArray<Mesh> {
    return RENDERER_PASSES.map((pass) => this.meshes[pass]);
  }

  /** The geometries only it draws: the places it stands and is about to stand. */
  public get owned(): ReadonlyArray<BufferGeometry> {
    return [this.instances, this.staged].flatMap((it: Nullable<SceneInstances>) => (it ? [it.geometry] : []));
  }

  /**
   * The places it stands over a geometry, kept while it is put with the same transforms over the same one, and built
   * once for the change that brings new ones.
   *
   * @param geometry - The geometry it names now.
   * @returns The places, or null for an object its matrix places once.
   */
  public toInstances(geometry: BufferGeometry): Nullable<SceneInstances> {
    const source: Maybe<IRendererInstances> = this.object.instances;

    if (!source) {
      return null;
    }

    if (this.instances?.isFor(geometry, source)) {
      return this.instances;
    }

    if (!this.staged?.isFor(geometry, source)) {
      // Staged for a change that never applied: nothing draws it.
      this.staged?.dispose();
      this.staged = new SceneInstances(geometry, source);
    }

    return this.staged;
  }

  /**
   * Points every mesh at what the object draws now, and shows each only in the scene of a pass it has a slot in.
   *
   * @param state - What it draws, or null for an object whose geometry is missing.
   * @param scenes - Each pass's scene.
   */
  public apply(state: Nullable<ISceneObjectState>, scenes: TPassRecord<Scene>): void {
    if (!state) {
      this.detach();

      return;
    }

    if (state.skeleton !== this.skeleton) {
      this.detach();
      this.meshes = toPassRecord(() => SceneObject.createMesh(state.skeleton));
      this.skeleton = state.skeleton;
    }

    // The places it stood before are nothing's once its meshes draw the new ones.
    if (this.instances && this.instances !== state.instances) {
      this.instances.dispose();
    }

    this.instances = state.instances;
    this.staged = null;

    const { drawRange, matrix } = this.object;

    state.geometry.setDrawRange(drawRange?.start ?? 0, drawRange?.count ?? Infinity);

    for (const pass of RENDERER_PASSES) {
      const mesh: Mesh = this.meshes[pass];
      const materials: Array<Material> = state.slots[pass];

      mesh.geometry = state.geometry;
      mesh.material = materials;

      if (matrix) {
        mesh.matrix.fromArray(matrix);
      } else {
        mesh.matrix.identity();
      }

      mesh.matrixWorldNeedsUpdate = true;

      if (materials.some((material: Material) => material !== HIDDEN_MATERIAL)) {
        scenes[pass].add(mesh);
      } else {
        mesh.removeFromParent();
      }
    }
  }

  /** Takes every mesh out of its scene. */
  public detach(): void {
    RENDERER_PASSES.forEach((pass) => this.meshes[pass].removeFromParent());
  }
}
