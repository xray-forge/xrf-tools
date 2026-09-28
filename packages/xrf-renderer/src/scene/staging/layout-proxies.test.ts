import { describe, expect, it } from "@jest/globals";
import {
  BufferAttribute,
  BufferGeometry,
  InstancedBufferAttribute,
  InstancedBufferGeometry,
  InstancedInterleavedBuffer,
  InterleavedBufferAttribute,
  IntType,
} from "three/webgpu";

import { LayoutProxies, toLayoutKey, toLayoutProxy } from "#/scene/staging/layout-proxies";

describe("toLayoutProxy", () => {
  it("lays a triangle out as the geometry is, holding none of its data", () => {
    const geometry: BufferGeometry = new BufferGeometry();
    const words: BufferAttribute = new BufferAttribute(new Uint32Array(4000), 2);

    words.gpuType = IntType;
    geometry.setAttribute("position", new BufferAttribute(new Float32Array(3000), 3));
    geometry.setAttribute("normal", new BufferAttribute(new Uint8Array(4000), 4, true));
    geometry.setAttribute("packedUv", words);
    geometry.setIndex(new BufferAttribute(new Uint32Array(6000), 1));

    const proxy: BufferGeometry = toLayoutProxy(geometry);
    const normal: BufferAttribute = proxy.getAttribute("normal") as BufferAttribute;

    expect(Object.keys(proxy.attributes)).toEqual(["position", "normal", "packedUv"]);
    expect(proxy.getAttribute("position").array).toBeInstanceOf(Float32Array);
    expect(proxy.getAttribute("position").count).toBe(3);
    expect(normal.array).toBeInstanceOf(Uint8Array);
    expect([normal.itemSize, normal.normalized]).toEqual([4, true]);
    expect((proxy.getAttribute("packedUv") as BufferAttribute).gpuType).toBe(IntType);
    expect(proxy.index?.array).toBeInstanceOf(Uint32Array);
    expect(proxy.index?.count).toBe(3);
  });

  // Three keys a build by its attributes: a stand-in built without what else its shader reads would be the build a
  // clustered draw of the same layout is drawn with.
  it("carries what a shader is built from beside the attributes", () => {
    const geometry: BufferGeometry = new BufferGeometry();

    geometry.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));
    geometry.userData = { clusters: {} };

    expect(toLayoutProxy(geometry).userData).toBe(geometry.userData);
  });

  // An instanced attribute steps once an instance, and interleaved columns share one buffer: both are the pipeline's.
  it("keeps instancing and interleaving, one buffer for the columns that shared one", () => {
    const geometry: InstancedBufferGeometry = new InstancedBufferGeometry();
    const columns: InstancedInterleavedBuffer = new InstancedInterleavedBuffer(new Float32Array(1600), 16, 1);

    geometry.setAttribute("position", new BufferAttribute(new Float32Array(30), 3));
    geometry.setAttribute("slot", new InstancedBufferAttribute(new Uint32Array(100), 1, false, 1));
    geometry.setAttribute("column0", new InterleavedBufferAttribute(columns, 4, 0));
    geometry.setAttribute("column1", new InterleavedBufferAttribute(columns, 4, 4));

    const proxy: BufferGeometry = toLayoutProxy(geometry);
    const first: InterleavedBufferAttribute = proxy.getAttribute("column0") as InterleavedBufferAttribute;
    const second: InterleavedBufferAttribute = proxy.getAttribute("column1") as InterleavedBufferAttribute;

    expect(proxy).toBeInstanceOf(InstancedBufferGeometry);
    expect(proxy.getAttribute("slot")).toBeInstanceOf(InstancedBufferAttribute);
    expect(first.data).toBeInstanceOf(InstancedInterleavedBuffer);
    expect(first.data).toBe(second.data);
    expect([first.data.stride, second.offset, first.data.array.length]).toEqual([16, 4, 48]);
    expect(proxy.index).toBeNull();
  });
});

describe("LayoutProxies", () => {
  function createGeometry(uv: number): BufferGeometry {
    const geometry: BufferGeometry = new BufferGeometry();

    geometry.setAttribute("position", new BufferAttribute(new Float32Array(30), 3));
    geometry.setAttribute("packedUv", new BufferAttribute(new Uint32Array(10 * uv), uv));

    return geometry;
  }

  // A proxy a stand-in would be thousands of buffers a level, each a page of the device's memory.
  it("shares one proxy between geometries of one layout", () => {
    const proxies: LayoutProxies = new LayoutProxies();
    const proxy: BufferGeometry = proxies.get(createGeometry(1));

    expect(proxies.get(createGeometry(1))).toBe(proxy);
    expect(proxies.get(createGeometry(2))).not.toBe(proxy);
  });

  // A tree's coordinate is two words where a sector's is one, under the same name, and its shader differs.
  it("tells layouts apart by format as well as by name", () => {
    expect(toLayoutKey(createGeometry(1))).not.toBe(toLayoutKey(createGeometry(2)));
    expect(toLayoutKey(createGeometry(1))).toBe(toLayoutKey(toLayoutProxy(createGeometry(1))));
  });
});
