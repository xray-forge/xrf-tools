import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Matrix4, Mesh, Scene, Skeleton, Sphere } from "three/webgpu";

import { IRendererInstances, IRendererObject } from "#/contract/scene/renderer-object";
import { ISurfaceMaterial } from "#/material/surface-material";
import { createPartGeometry } from "#/scene/geometry/part-geometry";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { SceneInstances } from "#/scene/object/scene-instances";
import { ISceneObjectState, isStaticDraw } from "#/scene/object/scene-object-state";
import { ScenePart } from "#/scene/object/scene-part";
import { TPassRecord } from "#/scene/pass-record";
import { StaticDraws } from "#/scene/static/static-draws";
import { IStaticRange } from "#/scene/static/static-range";
import { STATIC_LOD_IMPOSTOR_ROW, STATIC_NO_LOD } from "#/uniforms/static-draw-buffers";
import { CullView } from "#/visibility/cull-view";
import { EVisibility } from "#/visibility/visibility";

/**
 * One object a consumer put: a part per section of its geometry, each drawn in the pass its surface names.
 * The parts filling the G-buffer of an object neither instanced nor skinned are static draws: its geometry copied into
 * the arena of its layout, each part issued by its material's batch and culled on the GPU. The rest are drawn plainly
 * and culled here.
 */
export class SceneObject {
  /** The key it was put under, which a released object no longer answers to. */
  public readonly key: string;
  public object: IRendererObject;

  private readonly draws: StaticDraws;
  private parts: Array<ScenePart> = [];
  /** What the parts draw over plainly, the geometry put or the places it stands. */
  private drawn: Nullable<BufferGeometry> = null;
  /** The skeleton the parts' meshes are bound to, or null for rigid meshes. */
  private skeleton: Nullable<Skeleton> = null;
  /** The places its parts stand, or null for an object its matrix places once. */
  private instances: Nullable<SceneInstances> = null;
  /** The places it is about to stand in, built for the change it waits in. */
  private staged: Nullable<SceneInstances> = null;
  /** The geometry it holds a place in an arena for while any part is drawn statically, and that place. */
  private placedGeometry: Nullable<SceneGeometry> = null;
  private range: Nullable<IStaticRange> = null;
  /** Where its places start in the static draw buffers while its parts are instanced static draws, and how many. */
  private placeStart: Nullable<number> = null;
  private placeCount: number = 0;
  private readonly matrix: Matrix4 = new Matrix4();
  /** What all of it spans, in renderer space. */
  private readonly sphere: Sphere = new Sphere();
  /** The version of the view it was last culled against. */
  private culledAt: number = -1;

  /**
   * @param key - What it was put under.
   * @param object - What it is.
   * @param draws - The static draws its static parts are.
   */
  public constructor(key: string, object: IRendererObject, draws: StaticDraws) {
    this.key = key;
    this.object = object;
    this.draws = draws;
  }

  /** Every mesh of its parts, whether or not a scene holds it. */
  public get drawing(): ReadonlyArray<Mesh> {
    return this.parts.map((part: ScenePart) => part.mesh);
  }

  /** What stands in a scene for it: the meshes of the parts drawn plainly. */
  public get placed(): ReadonlyArray<Mesh> {
    return this.parts.map((part: ScenePart) => part.mesh).filter((mesh: Mesh) => mesh.parent);
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
   * Draws the object as it is put now: each part in the pass its surface names, where it stands.
   *
   * @param state - What it draws, or null for an object whose geometry is missing.
   * @param scenes - Each pass's scene.
   */
  public apply(state: Nullable<ISceneObjectState>, scenes: TPassRecord<Scene>): void {
    if (!state) {
      this.detach();

      return;
    }

    if (state.plain.drawn !== this.drawn) {
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
    this.show(state, scenes);

    // Culled afresh on the next frame, whatever the view.
    this.culledAt = -1;
  }

  /**
   * Shows what the view sees of the parts drawn plainly; the GPU culls the static ones.
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
      // Parts drawn as instanced static draws are culled place by place on the GPU.
      const plain: Array<ScenePart> = this.parts.filter((part: ScenePart) => !part.isStatic);

      if (plain.length) {
        const count: number = this.instances.cull(view);

        plain.forEach((part: ScenePart) => part.cullInstances(count));
      }

      return;
    }

    const whole: EVisibility = view.classifySphere(this.sphere);

    for (const part of this.parts) {
      if (!part.isStatic) {
        part.cull(
          whole === EVisibility.INSIDE ||
            (whole === EVisibility.INTERSECTS && view.classifySphere(part.sphere) !== EVisibility.OUTSIDE)
        );
      }
    }
  }

  /** Takes every part out of what draws it, letting its static slots and its place in an arena go. */
  public detach(): void {
    this.parts.forEach((part: ScenePart) => part.detach());
    this.unplace();
    this.freePlaces();
  }

  /** Lets everything it drew with go, for an object released. */
  public dispose(): void {
    this.release();
    this.unplace();
    this.freePlaces();
    this.instances?.dispose();
    this.staged?.dispose();
    this.instances = null;
    this.staged = null;
  }

  /** Lets its parts go, and with them the buffers they drew with. */
  private release(): void {
    this.parts.forEach((part: ScenePart) => {
      part.detach();
      part.geometry.dispose();
    });
    this.parts = [];
  }

  /** A part per section over what it draws now, letting the parts that drew before go. */
  private rebuild(state: ISceneObjectState): void {
    this.release();

    // The places it stood before are nothing's once its parts draw the new ones.
    if (this.instances && this.instances !== state.instances) {
      this.instances.dispose();
    }

    this.drawn = state.plain.drawn;
    this.instances = state.instances;
    this.parts = state.geometry.sections.map(
      (section: ISceneSection, index: number) =>
        new ScenePart(createPartGeometry(state.plain.drawn, section), index, section, state.skeleton, this.draws)
    );
  }

  /** Puts each part where its surface draws it: its material's batch for a static draw, its pass's scene otherwise. */
  private show(state: ISceneObjectState, scenes: TPassRecord<Scene>): void {
    const range: Nullable<IStaticRange> = state.surfaces.some((surface) => isStaticDraw(state, surface))
      ? this.place(state.geometry)
      : null;
    const placeStart: Nullable<number> =
      range && state.instances ? this.placeInstances(state.instances, state.lodStart) : null;
    let staticParts: number = 0;

    for (const part of this.parts) {
      const surface: Maybe<ISurfaceMaterial> = state.surfaces[part.section];

      part.narrow(this.object.drawRange);
      part.place(this.matrix);

      if (
        range &&
        isStaticDraw(state, surface) &&
        this.showStatic(part, surface, range, state.instances, placeStart, state.lodStart)
      ) {
        staticParts += 1;
      } else if (surface?.isImpostor) {
        // An impostor draws only where the LOD cull decides it does, which only a static draw is culled by.
        part.showPlain(null, null);
      } else {
        part.showPlain(surface?.material ?? null, surface ? scenes[surface.pass] : null, surface?.shadow ?? null);
      }
    }

    if (!staticParts) {
      this.unplace();
      this.freePlaces();
    }
  }

  /** Draws a part as a static draw of its kind: single by its slot, or instanced over its object's places. */
  private showStatic(
    part: ScenePart,
    surface: ISurfaceMaterial,
    range: IStaticRange,
    instances: Nullable<SceneInstances>,
    placeStart: Nullable<number>,
    lodStart: Nullable<number>
  ): boolean {
    if (!instances) {
      return !surface.isImpostor && part.showStatic(surface, range);
    }

    if (surface.isImpostor && lodStart === null) {
      return false;
    }

    return (
      placeStart !== null &&
      part.showListed(
        surface,
        range,
        placeStart,
        instances.placeSpheres,
        SceneObject.toRowLods(instances, lodStart, Boolean(surface.isImpostor))
      )
    );
  }

  /**
   * @param instances - An object's places.
   * @param lodStart - Where its set's impostors start, or null for none.
   * @param isImpostor - Whether the rows are the impostors' own draw rather than the trees of their clumps.
   * @returns Each place's row word for the LOD cull, or null where no place belongs to an impostor.
   */
  private static toRowLods(
    instances: SceneInstances,
    lodStart: Nullable<number>,
    isImpostor: boolean
  ): Nullable<Uint32Array> {
    const indices: Maybe<Int32Array> = instances.source.impostors?.indices;

    if (lodStart === null || !indices) {
      return null;
    }

    return Uint32Array.from(indices, (index: number) =>
      index < 0 ? STATIC_NO_LOD : (lodStart + index) | (isImpostor ? STATIC_LOD_IMPOSTOR_ROW : 0)
    );
  }

  /**
   * @param instances - The places it stands in now, as its matrix places them.
   * @param lodStart - Where its set's impostors start, or null for none.
   * @returns Where they start in the static draw buffers, written there first; null where there is no room.
   */
  private placeInstances(instances: SceneInstances, lodStart: Nullable<number>): Nullable<number> {
    if (this.placeStart !== null && this.placeCount !== instances.places) {
      this.freePlaces();
    }

    if (this.placeStart === null) {
      this.placeStart = this.draws.allocatePlaces(instances.places);
      this.placeCount = instances.places;
    }

    if (this.placeStart !== null) {
      this.draws.writePlaces(this.placeStart, instances.source, this.matrix, lodStart);
    }

    return this.placeStart;
  }

  private freePlaces(): void {
    if (this.placeStart !== null) {
      this.draws.freePlaces(this.placeStart, this.placeCount);
      this.placeStart = null;
      this.placeCount = 0;
    }
  }

  /**
   * @param geometry - The geometry its static parts draw.
   * @returns Where it sits in its arena, placed there first where this object held no place for it yet; null where
   *   the arena cannot hold it.
   */
  private place(geometry: SceneGeometry): Nullable<IStaticRange> {
    if (this.placedGeometry !== geometry) {
      // Taken before the old one goes, so an arena holding only the old one is not let go and made again at once.
      const range: Nullable<IStaticRange> = this.draws.acquire(geometry);

      this.unplace();
      this.placedGeometry = geometry;
      this.range = range;
    }

    return this.range;
  }

  private unplace(): void {
    if (this.placedGeometry) {
      this.draws.release(this.placedGeometry);
      this.placedGeometry = null;
      this.range = null;
    }
  }
}
