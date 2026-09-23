import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry } from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRoom } from "#/scene/static/static-room";
import { EVertexAttribute } from "#/shader/vertex-attribute";

function createBuffer(count: number, isIndexed: boolean = true): BufferGeometry {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute(
    "position",
    new BufferAttribute(
      new Float32Array(count * 3).map((_, it) => it),
      3
    )
  );

  if (isIndexed) {
    buffer.setIndex(
      new BufferAttribute(
        new Uint16Array(count).map((_, it) => count - 1 - it),
        1
      )
    );
  }

  return buffer;
}

/** Nothing placed after the geometry in question. */
function toNothingComing(): IStaticRoom {
  return { indices: 0, vertices: 0 };
}

describe("StaticArena", () => {
  it("holds geometries of one layout, telling each apart from one with another", () => {
    const withUv: BufferGeometry = createBuffer(3);

    withUv.setAttribute("uv", new BufferAttribute(new Float32Array(6), 2));

    expect(StaticArena.toSignature(createBuffer(3))).toBe(StaticArena.toSignature(createBuffer(6, false)));
    expect(StaticArena.toSignature(withUv)).not.toBe(StaticArena.toSignature(createBuffer(3)));
  });

  it("copies each geometry in after the last, its indices as stored, and a sequence for one without", () => {
    const arena: StaticArena = new StaticArena(createBuffer(3));
    const first: IStaticRange = arena.place(createBuffer(3), toNothingComing) as IStaticRange;
    const second: IStaticRange = arena.place(createBuffer(2, false), toNothingComing) as IStaticRange;
    const geometry: BufferGeometry = arena.createGeometry(EStaticDrawKind.SINGLE);

    expect([first.vertexStart, first.indexStart, second.vertexStart, second.indexStart]).toEqual([0, 0, 3, 3]);
    expect(Array.from((geometry.index as BufferAttribute).array.subarray(0, 5))).toEqual([2, 1, 0, 0, 1]);
    expect(Array.from(geometry.getAttribute("position").array.subarray(9, 12))).toEqual([0, 1, 2]);
    expect(geometry.getAttribute(EVertexAttribute.STATIC_SLOT).array[7]).toBe(7);
  });

  it("gives a freed geometry's room to the next, and says when it goes empty", () => {
    const arena: StaticArena = new StaticArena(createBuffer(3));
    const first: IStaticRange = arena.place(createBuffer(3), toNothingComing) as IStaticRange;

    arena.free(first);

    expect(arena.isEmpty).toBe(true);
    expect((arena.place(createBuffer(3), toNothingComing) as IStaticRange).vertexStart).toBe(0);
  });

  it("grows by replacing its buffers, keeping what they held, and counts each time", () => {
    const arena: StaticArena = new StaticArena(createBuffer(3));

    arena.place(createBuffer(3), toNothingComing);

    const generation: number = arena.generation;
    const range: IStaticRange = arena.place(createBuffer(1 << 17), toNothingComing) as IStaticRange;
    const geometry: BufferGeometry = arena.createGeometry(EStaticDrawKind.SINGLE);

    expect(arena.generation).toBeGreaterThan(generation);
    expect(range.vertexStart).toBe(3);
    expect(geometry.getAttribute("position").count).toBeGreaterThanOrEqual(3 + (1 << 17));
    expect(Array.from((geometry.index as BufferAttribute).array.subarray(0, 3))).toEqual([2, 1, 0]);
  });

  it("grows once for everything still to come, so what comes next fits without growing again", () => {
    const arena: StaticArena = new StaticArena(createBuffer(3));

    arena.place(createBuffer(3), () => ({ indices: 1 << 20, vertices: 1 << 20 }));

    const generation: number = arena.generation;

    arena.place(createBuffer(1 << 20), toNothingComing);

    expect(arena.generation).toBe(generation);
  });
});
