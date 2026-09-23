import { Nullable } from "@xrf/types";
import {
  InstancedBufferAttribute,
  InstancedBufferGeometry,
  InstancedInterleavedBuffer,
  InterleavedBufferAttribute,
  Matrix4,
  Sphere,
} from "three/webgpu";

import {
  IRendererInstances,
  RENDERER_FLOATS_PER_INSTANCE,
  RENDERER_HEMI_FLOATS_PER_INSTANCE,
} from "#/contract/scene/renderer-object";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { EVertexAttribute, INSTANCE_MATRIX_COLUMNS } from "#/shader/vertex-attribute";
import { CullView } from "#/visibility/cull-view";
import { collectVisibleInstances, FLOATS_PER_SPHERE, toInstanceSpheres } from "#/visibility/instance-spheres";
import { EVisibility } from "#/visibility/visibility";

/**
 * What an object stands in many places with: the geometry put, drawn once for every place the view sees, which the
 * object's parts draw section by section.
 * Every place is kept here; the ones seen are copied to the front of the instanced attributes the shaders read, and
 * only when that set changes, so a still view uploads nothing.
 */
export class SceneInstances {
  /** The geometry put under the object's key, whose buffers every part shares. */
  public readonly base: SceneGeometry;
  public readonly source: IRendererInstances;
  /** All of the geometry in every place seen, which the materials compile against. */
  public readonly geometry: InstancedBufferGeometry;

  private readonly count: number;
  /** The transforms drawn, the places seen first. */
  private readonly columns: InstancedInterleavedBuffer;
  private readonly hemi: Nullable<InstancedBufferAttribute>;
  /** Which place each drawn instance is, and how many of them are current. */
  private readonly drawn: Uint32Array;
  private drawnCount: number;
  /** Where the places seen are collected, before they are compared with what is drawn. */
  private readonly seen: Uint32Array;
  private visibleCount: number;
  private spheres: Float32Array = new Float32Array(0);
  /** What all of the places span together, in renderer space. */
  private readonly sphere: Sphere = new Sphere();
  private readonly placement: Matrix4 = new Matrix4();
  private culledAt: number = -1;

  public constructor(base: SceneGeometry, source: IRendererInstances) {
    this.base = base;
    this.source = source;
    this.count = source.transforms.length / RENDERER_FLOATS_PER_INSTANCE;
    this.columns = new InstancedInterleavedBuffer(source.transforms.slice(), RENDERER_FLOATS_PER_INSTANCE);
    this.hemi = source.hemi
      ? new InstancedBufferAttribute(source.hemi.slice(), RENDERER_HEMI_FLOATS_PER_INSTANCE)
      : null;
    this.drawn = Uint32Array.from({ length: this.count }, (_, index: number) => index);
    this.drawnCount = this.count;
    this.seen = new Uint32Array(this.count);
    this.visibleCount = this.count;
    this.geometry = this.createGeometry();
    this.place(this.placement);
  }

  /** How many places it stands in. */
  public get places(): number {
    return this.count;
  }

  /** Every place's sphere in renderer space, four floats each, as its object's matrix last placed it. */
  public get placeSpheres(): Float32Array {
    return this.spheres;
  }

  /** How many places the view last culled against sees. */
  public get visible(): number {
    return this.visibleCount;
  }

  /**
   * @param base - The geometry an object names now.
   * @param source - The places it names now.
   * @returns Whether these are still what it stands in.
   */
  public isFor(base: SceneGeometry, source: IRendererInstances): boolean {
    return this.base === base && this.source === source;
  }

  /**
   * @param matrix - What places every instance: the object's own matrix.
   */
  public place(matrix: Matrix4): void {
    if (this.spheres.length && this.placement.equals(matrix)) {
      return;
    }

    this.placement.copy(matrix);
    this.spheres = toInstanceSpheres(this.base.sphere, this.source.transforms, matrix);
    this.sphere.makeEmpty();

    const placed: Sphere = new Sphere();

    for (let at = 0; at < this.spheres.length; at += FLOATS_PER_SPHERE) {
      placed.center.set(this.spheres[at], this.spheres[at + 1], this.spheres[at + 2]);
      placed.radius = this.spheres[at + 3];
      this.sphere.union(placed);
    }

    this.culledAt = -1;
  }

  /**
   * Draws the places a view sees and no others.
   *
   * @param view - The view drawn for.
   * @returns How many places it sees.
   */
  public cull(view: CullView): number {
    if (view.version === this.culledAt) {
      return this.visibleCount;
    }

    this.culledAt = view.version;

    switch (view.classifySphere(this.sphere)) {
      case EVisibility.OUTSIDE:
        return this.show(0);

      case EVisibility.INSIDE:
        for (let index = 0; index < this.count; index += 1) {
          this.seen[index] = index;
        }

        return this.show(this.count);

      case EVisibility.INTERSECTS:
        return this.show(collectVisibleInstances(view, this.spheres, this.seen));
    }
  }

  /** Frees the buffers it drew with, the base's included: three frees whatever a disposed geometry names. */
  public dispose(): void {
    this.geometry.dispose();
  }

  private createGeometry(): InstancedBufferGeometry {
    const geometry: InstancedBufferGeometry = new InstancedBufferGeometry();
    const { buffer } = this.base;

    geometry.index = buffer.index;
    Object.entries(buffer.attributes).forEach(([name, attribute]) => geometry.setAttribute(name, attribute));
    INSTANCE_MATRIX_COLUMNS.forEach((column: string, index: number) =>
      geometry.setAttribute(column, new InterleavedBufferAttribute(this.columns, 4, index * 4))
    );

    if (this.hemi) {
      geometry.setAttribute(EVertexAttribute.INSTANCE_HEMI, this.hemi);
    }

    geometry.instanceCount = this.count;
    geometry.boundingSphere = this.sphere;

    return geometry;
  }

  /** Makes the first `count` collected places the ones drawn, uploading them only where they changed. */
  private show(count: number): number {
    if (count && !this.isDrawn(count)) {
      this.upload(count);
    }

    this.visibleCount = count;
    this.geometry.instanceCount = count;

    return count;
  }

  private isDrawn(count: number): boolean {
    if (count > this.drawnCount) {
      return false;
    }

    for (let index = 0; index < count; index += 1) {
      if (this.seen[index] !== this.drawn[index]) {
        return false;
      }
    }

    return true;
  }

  private upload(count: number): void {
    const { transforms, hemi } = this.source;
    const drawnTransforms: Float32Array = this.columns.array as Float32Array;

    for (let index = 0; index < count; index += 1) {
      const place: number = this.seen[index];

      this.drawn[index] = place;
      drawnTransforms.set(
        transforms.subarray(place * RENDERER_FLOATS_PER_INSTANCE, (place + 1) * RENDERER_FLOATS_PER_INSTANCE),
        index * RENDERER_FLOATS_PER_INSTANCE
      );

      if (this.hemi && hemi) {
        (this.hemi.array as Float32Array).set(
          hemi.subarray(place * RENDERER_HEMI_FLOATS_PER_INSTANCE, (place + 1) * RENDERER_HEMI_FLOATS_PER_INSTANCE),
          index * RENDERER_HEMI_FLOATS_PER_INSTANCE
        );
      }
    }

    this.drawnCount = count;
    this.columns.clearUpdateRanges();
    this.columns.addUpdateRange(0, count * RENDERER_FLOATS_PER_INSTANCE);
    this.columns.needsUpdate = true;

    if (this.hemi) {
      this.hemi.clearUpdateRanges();
      this.hemi.addUpdateRange(0, count * RENDERER_HEMI_FLOATS_PER_INSTANCE);
      this.hemi.needsUpdate = true;
    }
  }
}
