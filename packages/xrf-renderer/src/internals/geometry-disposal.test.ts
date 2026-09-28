import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, InterleavedBufferAttribute, WebGPURenderer } from "three/webgpu";

import { disposeSharingGeometry } from "#/internals/geometry-disposal";

type TGeometryAttribute = BufferAttribute | InterleavedBufferAttribute;

interface IGeometries {
  initGeometry(renderObject: object): void;
}

const { default: Geometries } = require("three/src/renderers/common/Geometries.js") as {
  default: new (attributes: { delete(attribute: TGeometryAttribute): void }, info: object) => IGeometries;
};

interface IRendererFixture {
  renderer: WebGPURenderer;
  geometries: IGeometries;
  /** Every buffer freed so far. */
  freed: Array<TGeometryAttribute>;
}

function createFixture(): IRendererFixture {
  const freed: Array<TGeometryAttribute> = [];
  const attributes: { delete(attribute: TGeometryAttribute): number } = {
    delete: (attribute: TGeometryAttribute): number => freed.push(attribute),
  };
  const info: { memory: { geometries: number } } = { memory: { geometries: 0 } };
  const geometries: IGeometries = new Geometries(attributes, info);

  return {
    freed,
    geometries,
    renderer: { _attributes: attributes, _geometries: geometries, info } as unknown as WebGPURenderer,
  };
}

function createSource(): BufferGeometry {
  const source: BufferGeometry = new BufferGeometry();

  source.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));
  source.setIndex(new BufferAttribute(new Uint16Array([0, 1, 2]), 1));

  return source;
}

function createSharing(source: BufferGeometry, own: BufferAttribute): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();

  geometry.setIndex(source.index);
  geometry.setAttribute("position", source.getAttribute("position"));
  geometry.setAttribute("own", own);

  return geometry;
}

describe("disposeSharingGeometry", () => {
  it("is needed: three frees the shared buffers of a geometry stripped of them, through a render object that went", () => {
    const { geometries, freed }: IRendererFixture = createFixture();
    const source: BufferGeometry = createSource();
    const geometry: BufferGeometry = createSharing(source, new BufferAttribute(new Float32Array(3), 1));
    const position: BufferAttribute = source.getAttribute("position") as BufferAttribute;

    geometries.initGeometry({ geometry, getAttributes: (): Array<TGeometryAttribute> => [position] });
    geometry.setIndex(null);
    geometry.deleteAttribute("position");
    geometry.dispose();

    expect(freed).toContain(position);
  });

  it("frees only what the geometry alone names, where the render object that first drew it went since", () => {
    const { renderer, geometries, freed }: IRendererFixture = createFixture();
    const source: BufferGeometry = createSource();
    const own: BufferAttribute = new BufferAttribute(new Float32Array(3), 1);
    const geometry: BufferGeometry = createSharing(source, own);

    // Disposed since, its material gone: its list of what it drew is still the whole geometry.
    geometries.initGeometry({
      geometry,
      getAttributes: (): Array<TGeometryAttribute> => [source.getAttribute("position") as BufferAttribute, own],
    });
    disposeSharingGeometry(renderer, geometry, source);

    expect(freed).toEqual([own]);
    expect(renderer.info.memory.geometries).toBe(0);
  });

  it("frees the line index three built for a geometry drawn as a wireframe", () => {
    const { renderer, geometries, freed }: IRendererFixture = createFixture();
    const source: BufferGeometry = createSource();
    const geometry: BufferGeometry = createSharing(source, new BufferAttribute(new Float32Array(3), 1));
    const wireframe: BufferAttribute = new BufferAttribute(new Uint16Array(6), 1);

    geometries.initGeometry({ geometry, getAttributes: (): Array<TGeometryAttribute> => [] });
    (geometries as unknown as { wireframes: WeakMap<BufferGeometry, BufferAttribute> }).wireframes.set(
      geometry,
      wireframe
    );
    disposeSharingGeometry(renderer, geometry, source);

    expect(freed).toContain(wireframe);
    expect(freed).not.toContain(source.index);
  });

  it("disposes a geometry three never drew, freeing its own buffers", () => {
    const { renderer, freed }: IRendererFixture = createFixture();
    const source: BufferGeometry = createSource();
    const own: BufferAttribute = new BufferAttribute(new Float32Array(3), 1);
    const geometry: BufferGeometry = createSharing(source, own);
    let events: number = 0;

    geometry.addEventListener("dispose", () => (events += 1));
    disposeSharingGeometry(renderer, geometry, source);

    expect(freed).toEqual([own]);
    expect(events).toBe(1);
  });
});
