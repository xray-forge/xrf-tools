import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Matrix4, Mesh, Scene, Skeleton, Sphere } from "three/webgpu";

import { IRendererInstances, IRendererObject } from "#/contract/scene/renderer-object";
import { createPartGeometry } from "#/scene/geometry/part-geometry";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { SceneInstances } from "#/scene/object/scene-instances";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { ScenePart } from "#/scene/object/scene-part";
import { TPassRecord } from "#/scene/pass-record";
import { CullView } from "#/visibility/cull-view";
import { EVisibility } from "#/visibility/visibility";

/**
 * One object a consumer put: a part per section of its geometry, each drawn in the pass its surface names and culled
 * on its own.
 */
export class SceneObject {
  /** The key it was put under, which a released object no longer answers to. */
  public readonly key: string;
  public object: IRendererObject;

  private parts: Array<ScenePart> = [];
  /** What the parts draw over, the geometry put or the places it stands. */
  private drawn: Nullable<BufferGeometry> = null;
  /** The skeleton the parts' meshes are bound to, or null for rigid meshes. */
  private skeleton: Nullable<Skeleton> = null;
  /** The places its parts stand, or null for an object its matrix places once. */
  private instances: Nullable<SceneInstances> = null;
  /** The places it is about to stand in, built for the change it waits in. */
  private staged: Nullable<SceneInstances> = null;
  private readonly matrix: Matrix4 = new Matrix4();
  /** What all of it spans, in renderer space. */
  private readonly sphere: Sphere = new Sphere();
  /** The version of the view it was last culled against. */
  private culledAt: number = -1;

  public constructor(key: string, object: IRendererObject) {
    this.key = key;
    this.object = object;
  }

  /** Every mesh drawing it, whether or not a scene holds it. */
  public get drawing(): ReadonlyArray<Mesh> {
    return this.parts.map((part: ScenePart) => part.mesh);
  }

  /** The geometries only it draws with: its parts, and the places it stands and is about to stand. */
  public get owned(): ReadonlyArray<BufferGeometry> {
    return [
      ...this.parts.map((part: ScenePart) => part.geometry),
      ...[this.instances, this.staged].flatMap((it: Nullable<SceneInstances>) => (it ? [it.geometry] : [])),
    ];
  }

  /**
   * The places it stands over a geometry, kept while it is put with the same transforms over the same one, and built
   * once for the change that brings new ones.
   *
   * @param geometry - The geometry it names now.
   * @returns The places, or null for an object its matrix places once.
   */
  public toInstances(geometry: SceneGeometry): Nullable<SceneInstances> {
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
   * Draws the object as it is put now: each part in the scene of the pass its surface names, where it stands.
   *
   * @param state - What it draws, or null for an object whose geometry is missing.
   * @param scenes - Each pass's scene.
   */
  public apply(state: Nullable<ISceneObjectState>, scenes: TPassRecord<Scene>): void {
    if (!state) {
      this.parts.forEach((part: ScenePart) => part.detach());

      return;
    }

    if (state.drawn !== this.drawn) {
      this.rebuild(state);
    } else if (state.skeleton !== this.skeleton) {
      this.parts.forEach((part: ScenePart) => part.remesh(state.skeleton));
    }

    this.skeleton = state.skeleton;
    this.staged = null;

    if (this.object.matrix) {
      this.matrix.fromArray(this.object.matrix);
    } else {
      this.matrix.identity();
    }

    this.instances?.place(this.matrix);
    this.sphere.copy(state.geometry.sphere).applyMatrix4(this.matrix);

    for (const part of this.parts) {
      const surface = state.surfaces[part.section];

      part.narrow(this.object.drawRange);
      part.place(this.matrix);
      part.show(surface?.material ?? null, surface ? scenes[surface.pass] : null);
    }

    // Culled afresh on the next frame, whatever the view.
    this.culledAt = -1;
  }

  /**
   * Shows the parts the view sees and hides the rest.
   *
   * @param view - The view drawn for.
   */
  public cull(view: CullView): void {
    if (view.version === this.culledAt) {
      return;
    }

    this.culledAt = view.version;

    // Measured against its bind pose, a skinned object would be culled by a motion reaching outside it.
    if (this.skeleton) {
      this.parts.forEach((part: ScenePart) => part.cull(true));

      return;
    }

    if (this.instances) {
      const count: number = this.instances.cull(view);

      this.parts.forEach((part: ScenePart) => part.cullInstances(count));

      return;
    }

    const whole: EVisibility = view.classifySphere(this.sphere);

    for (const part of this.parts) {
      part.cull(
        whole === EVisibility.INSIDE ||
          (whole === EVisibility.INTERSECTS && view.classifySphere(part.sphere) !== EVisibility.OUTSIDE)
      );
    }
  }

  /** Takes every part out of its scene. */
  public detach(): void {
    this.parts.forEach((part: ScenePart) => part.detach());
  }

  /** A part per section over what it draws now, letting the parts that drew before go. */
  private rebuild(state: ISceneObjectState): void {
    this.parts.forEach((part: ScenePart) => {
      part.detach();
      part.geometry.dispose();
    });

    // The places it stood before are nothing's once its parts draw the new ones.
    if (this.instances && this.instances !== state.instances) {
      this.instances.dispose();
    }

    this.drawn = state.drawn;
    this.instances = state.instances;
    this.parts = state.geometry.sections.map(
      (section: ISceneSection, index: number) =>
        new ScenePart(createPartGeometry(state.drawn, section), index, section, state.skeleton)
    );
  }
}
