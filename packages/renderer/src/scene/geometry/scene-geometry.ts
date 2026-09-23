import { BufferAttribute, BufferGeometry, Sphere, Vector3 } from "three/webgpu";

import { IRendererBounds, IRendererGeometry, IRendererGeometryGroup } from "#/contract/scene/renderer-geometry";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { toSectionSphere } from "#/scene/geometry/section-sphere";
import { EVertexAttribute } from "#/shader/vertex-attribute";

/**
 * A geometry a consumer put: the buffers every object drawing it shares, over the very arrays that crossed, and the
 * sections those objects draw and cull it by.
 */
export class SceneGeometry {
  /** A sphere as the contract states one. */
  private static toSphere(bounds: IRendererBounds): Sphere {
    return new Sphere(new Vector3(...bounds.center), bounds.radius);
  }

  private static createBuffer(geometry: IRendererGeometry): BufferGeometry {
    const buffer: BufferGeometry = new BufferGeometry();

    buffer.setAttribute("position", new BufferAttribute(geometry.position, 3));

    if (geometry.uv) {
      buffer.setAttribute("uv", new BufferAttribute(geometry.uv, 2));
    }

    if (geometry.uv1) {
      buffer.setAttribute("uv1", new BufferAttribute(geometry.uv1, 2));
    }

    if (geometry.tangent) {
      buffer.setAttribute("tangent", new BufferAttribute(geometry.tangent, 3));
    }

    if (geometry.binormal) {
      buffer.setAttribute(EVertexAttribute.BINORMAL, new BufferAttribute(geometry.binormal, 3));
    }

    if (geometry.skinIndices && geometry.skinWeights) {
      buffer.setAttribute("skinIndex", new BufferAttribute(geometry.skinIndices, 4));
      buffer.setAttribute("skinWeight", new BufferAttribute(geometry.skinWeights, 4));
    }

    if (geometry.hemi) {
      buffer.setAttribute(EVertexAttribute.HEMI, new BufferAttribute(geometry.hemi, 1));
    }

    if (geometry.index) {
      buffer.setIndex(new BufferAttribute(geometry.index, 1));
    }

    if (geometry.normal) {
      buffer.setAttribute("normal", new BufferAttribute(geometry.normal, 3));
    } else {
      // The G-buffer stores a normal for every pixel; a geometry without one is given the one its faces imply.
      buffer.computeVertexNormals();
    }

    if (geometry.bounds) {
      buffer.boundingSphere = SceneGeometry.toSphere(geometry.bounds);
    } else {
      buffer.computeBoundingSphere();
    }

    return buffer;
  }

  /** Its ranges, or one range drawing all of it for a geometry stating none, each bounded. */
  private static toSections(geometry: IRendererGeometry): Array<ISceneSection> {
    const groups: ReadonlyArray<IRendererGeometryGroup> = geometry.groups.length
      ? geometry.groups
      : [{ count: geometry.index ? geometry.index.length : geometry.position.length / 3, slot: 0, start: 0 }];

    return groups.map(({ start, count, slot, bounds }: IRendererGeometryGroup) => ({
      count,
      slot,
      sphere: bounds
        ? SceneGeometry.toSphere(bounds)
        : toSectionSphere(geometry.position, geometry.index ?? null, start, count),
      start,
    }));
  }

  public readonly buffer: BufferGeometry;
  public readonly sections: ReadonlyArray<ISceneSection>;

  public constructor(geometry: IRendererGeometry) {
    this.buffer = SceneGeometry.createBuffer(geometry);
    this.sections = SceneGeometry.toSections(geometry);
  }

  /** What all of it spans, in its own space. */
  public get sphere(): Sphere {
    return this.buffer.boundingSphere as Sphere;
  }

  public dispose(): void {
    this.buffer.dispose();
  }
}
