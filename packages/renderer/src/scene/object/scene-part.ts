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
import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { createSceneMesh } from "#/scene/object/scene-mesh";
import { StaticDraws } from "#/scene/static/static-draws";
import { IStaticRange } from "#/scene/static/static-range";
import { STATIC_NO_BAND, toStaticBandWord } from "#/uniforms/static-draw-buffers";

/**
 * One section of an object as the frame draws it: a mesh over the section's range, bounded where it stands.
 * Drawn plainly it sits in its pass's scene and the CPU culls it; drawn statically it is a slot its material's batch
 * issues an indirect draw of, and the GPU culls it.
 */
export class ScenePart {
  /** Its own geometry over the object's buffers, which it disposes. */
  public readonly geometry: BufferGeometry;
  /** Its section's position among the geometry's, which its surface is found by. */
  public readonly section: number;
  /** What it spans in renderer space. */
  public readonly sphere: Sphere = new Sphere();

  private readonly source: ISceneSection;
  private readonly draws: StaticDraws;
  private readonly matrix: Matrix4 = new Matrix4();
  private currentMesh: Mesh;
  private skeleton: Nullable<Skeleton>;
  /** Its twin in the cascades' plain casters while it is drawn plainly by a surface that casts, made the first time. */
  private shadowMesh: Nullable<Mesh> = null;
  /** The range the object's narrowing leaves it. */
  private start: number;
  private count: number;
  /** Its slots while drawn statically: one, or one a band for a progressive mesh drawn instanced. */
  private slots: Array<number> = [];

  /**
   * @param geometry - Its part geometry.
   * @param section - Its section's position among the geometry's.
   * @param source - The section itself.
   * @param skeleton - What its mesh is skinned to, or null.
   * @param draws - The static draws it is one of while drawn statically.
   */
  public constructor(
    geometry: BufferGeometry,
    section: number,
    source: ISceneSection,
    skeleton: Nullable<Skeleton>,
    draws: StaticDraws
  ) {
    this.geometry = geometry;
    this.section = section;
    this.source = source;
    this.draws = draws;
    this.start = source.start;
    this.count = source.count;
    this.skeleton = skeleton;
    this.currentMesh = createSceneMesh(geometry, skeleton);
  }

  /** Its mesh, which draws it plainly. */
  public get mesh(): Mesh {
    return this.currentMesh;
  }

  /** Whether it is drawn statically. */
  public get isStatic(): boolean {
    return this.slots.length > 0;
  }

  /**
   * @param skeleton - What its mesh is skinned to now; a new mesh over the same geometry, shown nowhere yet.
   */
  public remesh(skeleton: Nullable<Skeleton>): void {
    this.detach();
    this.skeleton = skeleton;
    this.currentMesh = createSceneMesh(this.geometry, skeleton);
    this.shadowMesh = null;
  }

  /**
   * @param range - The index range the object narrows its geometry to, if any.
   */
  public narrow(range: Maybe<NonNullable<IRendererObject["drawRange"]>>): void {
    const start: number = Math.max(this.source.start, range?.start ?? 0);
    const end: number = Math.min(this.source.start + this.source.count, range ? range.start + range.count : Infinity);

    this.start = start;
    this.count = Math.max(0, end - start);
    this.geometry.setDrawRange(this.start, this.count);
  }

  /**
   * @param matrix - Where the object stands.
   */
  public place(matrix: Matrix4): void {
    this.matrix.copy(matrix);
    this.sphere.copy(this.source.sphere).applyMatrix4(matrix);
    this.currentMesh.matrix.copy(matrix);
    this.currentMesh.updateMatrixWorld(true);

    if (this.shadowMesh) {
      this.shadowMesh.matrix.copy(matrix);
      this.shadowMesh.updateMatrixWorld(true);
    }
  }

  /**
   * Draws it plainly, in its pass's scene, letting any static slot it held go.
   *
   * @param material - What draws it, or null for a section whose surface is missing.
   * @param scene - The scene of the pass drawing that material.
   * @param shadow - What draws it into the sun's cascades, or null for a surface that casts none.
   */
  public showPlain(material: Nullable<Material>, scene: Nullable<Scene>, shadow: Nullable<Material> = null): void {
    this.free();

    if (material && scene) {
      this.currentMesh.material = material;
      scene.add(this.currentMesh);
    } else {
      this.currentMesh.removeFromParent();
    }

    this.showShadow(material && scene ? shadow : null);
  }

  /**
   * Draws it as a static draw of its material's batch, taking its mesh out of any scene.
   *
   * @param surface - What draws it.
   * @param range - Where its object's geometry sits in its arena.
   * @returns Whether it is drawn so; not where every slot is taken, and it has to be drawn plainly.
   */
  public showStatic(surface: ISurfaceMaterial, range: IStaticRange): boolean {
    if (!this.takeSlots(1)) {
      return false;
    }

    this.draws.draw(this.slots[0], surface, range, this.start, this.count, this.sphere, this.matrix);
    this.currentMesh.removeFromParent();
    this.showShadow(null);

    return true;
  }

  /**
   * Draws it as an instanced static draw of its material's batch: once for every place the instance cull keeps. A
   * progressive mesh is a draw a band, the first its own range, each keeping the places whose detail falls in it.
   *
   * @param surface - What draws it.
   * @param range - Where its object's geometry sits in its arena.
   * @param placeStart - Where its object's places start.
   * @param spheres - Each place's sphere in renderer space.
   * @param lods - Each place's impostor as its row names it, or null where none stands in for any.
   * @returns Whether it is drawn so; not where there is no room for it, and it has to be drawn plainly.
   */
  public showListed(
    surface: ISurfaceMaterial,
    range: IStaticRange,
    placeStart: number,
    spheres: Float32Array,
    lods: Nullable<Uint32Array> = null
  ): boolean {
    const progressive = this.source.progressive;
    const bands: number = progressive?.bands.length ?? 1;

    if (!this.takeSlots(bands)) {
      return false;
    }

    for (let band = 0; band < bands; band += 1) {
      const [start, count] =
        band && progressive ? [progressive.bands[band].start, progressive.bands[band].count] : [this.start, this.count];
      const word: number = progressive ? toStaticBandWord(band, bands, progressive.windows) : STATIC_NO_BAND;

      if (!this.draws.drawListed(this.slots[band], surface, range, start, count, placeStart, spheres, lods, word)) {
        this.free();

        return false;
      }
    }

    this.currentMesh.removeFromParent();
    this.showShadow(null);

    return true;
  }

  /**
   * @param isSeen - Whether the view drawn for sees it, for a part drawn plainly. Its twin casts whether or not the
   *   view sees it: what stands out of view still shades what is in it.
   */
  public cull(isSeen: boolean): void {
    this.currentMesh.visible = this.count > 0 && isSeen;

    if (this.shadowMesh) {
      this.shadowMesh.visible = this.count > 0;
    }
  }

  /**
   * @param count - How many of its instances the view sees.
   */
  public cullInstances(count: number): void {
    (this.geometry as InstancedBufferGeometry).instanceCount = count;
    this.cull(count > 0);
  }

  /** Takes it out of whatever draws it, and lets its slot go. */
  public detach(): void {
    this.free();
    this.currentMesh.removeFromParent();
    this.showShadow(null);
  }

  /** Stands its twin in the cascades' plain casters, drawn by the shadow material given, or takes it out for none. */
  private showShadow(material: Nullable<Material>): void {
    if (!material) {
      this.shadowMesh?.removeFromParent();

      return;
    }

    if (!this.shadowMesh) {
      this.shadowMesh = createSceneMesh(this.geometry, this.skeleton, material);
      this.shadowMesh.matrix.copy(this.matrix);
      this.shadowMesh.updateMatrixWorld(true);
    }

    this.shadowMesh.material = material;
    this.shadowMesh.visible = this.count > 0;
    this.draws.plainCasters.add(this.shadowMesh);
  }

  /**
   * @param count - Slots it draws with from now on.
   * @returns Whether it holds that many; not where the pool ran out, having let every one go.
   */
  private takeSlots(count: number): boolean {
    while (this.slots.length > count) {
      this.draws.free(this.slots.pop() as number);
    }

    while (this.slots.length < count) {
      const slot: Nullable<number> = this.draws.allocate();

      if (slot === null) {
        this.free();

        return false;
      }

      this.slots.push(slot);
    }

    return true;
  }

  private free(): void {
    this.slots.forEach((slot: number) => this.draws.free(slot));
    this.slots = [];
  }
}
