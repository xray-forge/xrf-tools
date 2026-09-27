import { Maybe } from "@xrf/types";
import {
  BufferAttribute,
  BufferGeometry,
  InstancedBufferAttribute,
  InstancedBufferGeometry,
  InstancedInterleavedBuffer,
  InterleavedBuffer,
  InterleavedBufferAttribute,
  TypedArray,
} from "three/webgpu";

/** Vertices a proxy holds: one triangle, the least a mesh draws. */
const PROXY_VERTICES: number = 3;

type TGeometryAttribute = BufferAttribute | InterleavedBufferAttribute;

/**
 * One proxy a vertex layout, shared by every stand-in drawing it: three uploads each attribute of a geometry as a
 * buffer of its own, and a device rounds every buffer up to a page of memory, so a proxy each would cost far more than
 * the geometry it stands in for.
 */
export class LayoutProxies {
  private readonly bySource: WeakMap<BufferGeometry, BufferGeometry> = new WeakMap();
  private readonly byLayout: Map<string, BufferGeometry> = new Map();

  /**
   * @param geometry - A geometry a stand-in is drawn for.
   * @returns The proxy of its layout, made the first time the layout is asked for.
   */
  public get(geometry: BufferGeometry): BufferGeometry {
    let proxy: Maybe<BufferGeometry> = this.bySource.get(geometry);

    if (!proxy) {
      const key: string = toLayoutKey(geometry);

      proxy = this.byLayout.get(key);

      if (!proxy) {
        proxy = toLayoutProxy(geometry);
        this.byLayout.set(key, proxy);
      }

      this.bySource.set(geometry, proxy);
    }

    return proxy;
  }

  public dispose(): void {
    this.byLayout.forEach((proxy: BufferGeometry) => proxy.dispose());
    this.byLayout.clear();
  }
}

/**
 * @param geometry - A geometry.
 * @returns Everything of its layout a shader or a pipeline is built from: each attribute's name, kind, format, step and
 *   place in an interleaved buffer, and the index's width.
 */
export function toLayoutKey(geometry: BufferGeometry): string {
  const buffers: Array<InterleavedBuffer> = [];
  const attributes: Array<string> = Object.keys(geometry.attributes)
    .sort()
    .map((name: string) => {
      const attribute: TGeometryAttribute = geometry.attributes[name] as TGeometryAttribute;

      if (attribute instanceof InterleavedBufferAttribute) {
        const { data } = attribute;
        const buffer: number = buffers.includes(data) ? buffers.indexOf(data) : buffers.push(data) - 1;
        const step: number = data instanceof InstancedInterleavedBuffer ? data.meshPerAttribute : 0;

        return [
          name,
          buffer,
          data.array.constructor.name,
          data.stride,
          step,
          attribute.offset,
          attribute.itemSize,
          attribute.normalized,
        ].join(":");
      }

      const step: number = attribute instanceof InstancedBufferAttribute ? attribute.meshPerAttribute : 0;

      return [
        name,
        attribute.array.constructor.name,
        attribute.itemSize,
        attribute.normalized,
        attribute.gpuType,
        step,
      ].join(":");
    });

  return [
    geometry instanceof InstancedBufferGeometry ? "instanced" : "",
    geometry.index?.array.constructor.name ?? "",
    ...attributes,
  ].join("|");
}

/**
 * A triangle laid out as a geometry is: every attribute of the same name, kind, format and step, interleaved as it is,
 * and an index of the same width. A mesh over it builds the shaders and pipelines a mesh over the geometry draws with,
 * holding none of its data.
 *
 * @param geometry - The geometry to take the layout of.
 * @returns A new geometry of that layout.
 */
export function toLayoutProxy(geometry: BufferGeometry): BufferGeometry {
  const proxy: BufferGeometry =
    geometry instanceof InstancedBufferGeometry ? new InstancedBufferGeometry() : new BufferGeometry();
  const buffers: Map<InterleavedBuffer, InterleavedBuffer> = new Map();

  if (proxy instanceof InstancedBufferGeometry) {
    proxy.instanceCount = 1;
  }

  for (const [name, attribute] of Object.entries(geometry.attributes)) {
    proxy.setAttribute(name, toProxyAttribute(attribute as TGeometryAttribute, buffers));
  }

  if (geometry.index) {
    proxy.setIndex(new BufferAttribute(toArray(geometry.index.array, PROXY_VERTICES), 1));
  }

  // What a shader is built from beside the attributes, such as the arena a clustered draw reads its vertices from:
  // three keys a build by the attributes alone, so a stand-in built without it would be the build its objects draw.
  proxy.userData = geometry.userData;

  return proxy;
}

function toProxyAttribute(
  attribute: TGeometryAttribute,
  buffers: Map<InterleavedBuffer, InterleavedBuffer>
): TGeometryAttribute {
  if (attribute instanceof InterleavedBufferAttribute) {
    let data: Maybe<InterleavedBuffer> = buffers.get(attribute.data);

    if (!data) {
      const array: TypedArray = toArray(attribute.data.array, attribute.data.stride * PROXY_VERTICES);

      data =
        attribute.data instanceof InstancedInterleavedBuffer
          ? new InstancedInterleavedBuffer(array, attribute.data.stride, attribute.data.meshPerAttribute)
          : new InterleavedBuffer(array, attribute.data.stride);
      buffers.set(attribute.data, data);
    }

    return new InterleavedBufferAttribute(data, attribute.itemSize, attribute.offset, attribute.normalized);
  }

  const array: TypedArray = toArray(attribute.array, attribute.itemSize * PROXY_VERTICES);
  const proxy: BufferAttribute =
    attribute instanceof InstancedBufferAttribute
      ? new InstancedBufferAttribute(array, attribute.itemSize, attribute.normalized, attribute.meshPerAttribute)
      : new BufferAttribute(array, attribute.itemSize, attribute.normalized);

  // Three reads an integer attribute's type off `gpuType`, which the vertex format follows.
  proxy.gpuType = attribute.gpuType;

  return proxy;
}

/** An empty array of the same type as another. */
function toArray(array: TypedArray | ArrayLike<number>, length: number): TypedArray {
  return new (array.constructor as new (length: number) => TypedArray)(length);
}
