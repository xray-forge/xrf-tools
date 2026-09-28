import { describe, expect, it, jest } from "@jest/globals";
import { storage } from "three/tsl";
import { BufferAttribute, BufferGeometry, StorageBufferAttribute } from "three/webgpu";

import { toClusterSource } from "#/geometry/cluster-source";
import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { StaticArena } from "#/scene/static/static-arena";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRoom } from "#/scene/static/static-room";
import { StorageRetirement } from "#/uniforms/storage-retirement";

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

function createArena(
  buffer: BufferGeometry = createBuffer(3),
  retirement: StorageRetirement = new StorageRetirement()
): StaticArena {
  return new StaticArena(
    buffer,
    storage(new StorageBufferAttribute(new Uint32Array(2), 2), "uvec2", 1).toReadOnly(),
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly(),
    retirement
  );
}

function createRetirement(): { retirement: StorageRetirement; retired: Array<BufferAttribute> } {
  const retirement: StorageRetirement = new StorageRetirement();
  const retired: Array<BufferAttribute> = [];

  jest
    .spyOn(retirement, "retire")
    .mockImplementation((attributes: Iterable<BufferAttribute>) => void retired.push(...attributes));

  return { retired, retirement };
}

function toNothingComing(): IStaticRoom {
  return { indices: 0, vertices: 0 };
}

/** Room a buffer of the default limit holds, which no test reaches. */
const LIMITS: IStaticRoom = { indices: 1 << 25, vertices: 1 << 23 };

function toWords(arena: StaticArena): Uint32Array {
  return arena.wordNode.value.array as Uint32Array;
}

function toIndices(arena: StaticArena): Uint32Array {
  return arena.indexNode.value.array as Uint32Array;
}

describe("StaticArena", () => {
  it("holds geometries of one layout, telling each apart from one with another", () => {
    const withUv: BufferGeometry = createBuffer(3);

    withUv.setAttribute("uv", new BufferAttribute(new Float32Array(6), 2));

    expect(StaticArena.toSignature(createBuffer(3))).toBe(StaticArena.toSignature(createBuffer(6, false)));
    expect(StaticArena.toSignature(withUv)).not.toBe(StaticArena.toSignature(createBuffer(3)));
  });

  // Skin links are sixteen-bit: a skinned geometry is drawn plainly, as its bones move it.
  it("stores no layout an attribute of which has no word format", () => {
    const skinned: BufferGeometry = createBuffer(3);

    skinned.setAttribute("skinIndex", new BufferAttribute(new Uint16Array(12), 4));

    expect(StaticArena.toSignature(skinned)).toBeNull();
  });

  it("copies each geometry's vertices in after the last, a vertex its attributes' words one after another", () => {
    const packed: BufferGeometry = createBuffer(2, false);

    packed.setAttribute(
      EVertexAttribute.PACKED_NORMAL,
      new BufferAttribute(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]), 4, true)
    );

    const arena: StaticArena = createArena(packed);

    arena.place(packed, toNothingComing, LIMITS);

    const second: IStaticRange = arena.place(packed, toNothingComing, LIMITS) as IStaticRange;
    const words: Uint32Array = toWords(arena);
    const floats: Float32Array = new Float32Array(words.buffer);

    // The normal's four bytes one word, before the position's three floats: attributes sorted by name.
    expect(arena.stride).toBe(4);
    expect(words[0]).toBe(0x04030201);
    expect(Array.from(floats.subarray(1, 4))).toEqual([0, 1, 2]);
    expect(second.vertexStart).toBe(2);
    expect(words[second.vertexStart * 4 + 4]).toBe(0x08070605);
  });

  it("copies each geometry's indices in after the last, as stored, and a sequence for one without", () => {
    const arena: StaticArena = createArena();
    const first: IStaticRange = arena.place(createBuffer(3), toNothingComing, LIMITS) as IStaticRange;
    const second: IStaticRange = arena.place(createBuffer(2, false), toNothingComing, LIMITS) as IStaticRange;

    expect([first.vertexStart, first.indexStart, second.vertexStart, second.indexStart]).toEqual([0, 0, 3, 3]);
    expect(Array.from(toIndices(arena).subarray(0, 5))).toEqual([2, 1, 0, 0, 1]);
  });

  it("gives a freed geometry's room to the next", () => {
    const arena: StaticArena = createArena();
    const first: IStaticRange = arena.place(createBuffer(3), toNothingComing, LIMITS) as IStaticRange;

    arena.free(first);

    expect((arena.place(createBuffer(3), toNothingComing, LIMITS) as IStaticRange).vertexStart).toBe(0);
  });

  // The nodes stay, pointed at the new buffers, so no shader over the arena is built again.
  it("grows by replacing its short buffer behind the same node, keeping what it held, and retires the old", () => {
    const { retired, retirement } = createRetirement();
    const arena: StaticArena = createArena(createBuffer(3), retirement);

    arena.place(createBuffer(3), toNothingComing, LIMITS);

    const generation: number = arena.generation;
    const node = arena.wordNode;
    const words: BufferAttribute = arena.wordNode.value;
    const index: BufferAttribute = arena.indexNode.value;
    const range: IStaticRange = arena.place(createBuffer(1 << 17), toNothingComing, LIMITS) as IStaticRange;

    expect(arena.generation).toBeGreaterThan(generation);
    expect(arena.wordNode).toBe(node);
    expect(range.vertexStart).toBe(3);
    expect(toWords(arena).length).toBeGreaterThanOrEqual((3 + (1 << 17)) * arena.stride);
    expect(Array.from(toIndices(arena).subarray(0, 3))).toEqual([2, 1, 0]);
    // The indices had room: only the words were copied.
    expect(retired).toContain(words);
    expect(retired).not.toContain(index);
    expect(arena.indexNode.value).toBe(index);
  });

  it("grows once for everything still to come, so what comes next fits without growing again", () => {
    const arena: StaticArena = createArena();

    arena.place(createBuffer(3), () => ({ indices: 1 << 20, vertices: 1 << 20 }), LIMITS);

    const generation: number = arena.generation;

    arena.place(createBuffer(1 << 20), toNothingComing, LIMITS);

    expect(arena.generation).toBe(generation);
  });

  // A refused geometry copied the whole arena for nothing, every time one was refused.
  it("holds no geometry past the device's limit, and copies nothing for it", () => {
    const { retired, retirement } = createRetirement();
    const arena: StaticArena = createArena(createBuffer(3), retirement);
    const limits: IStaticRoom = { indices: 1 << 25, vertices: 1 << 16 };

    arena.place(createBuffer(3), toNothingComing, limits);

    const generation: number = arena.generation;

    retired.length = 0;

    expect(arena.place(createBuffer(1 << 16), toNothingComing, limits)).toBeNull();
    expect(arena.generation).toBe(generation);
    expect(retired).toEqual([]);
  });

  // A batch's geometry keys the programs it is drawn with by the arena's layout and mark, as the prototype a material
  // compiles against does, and a shader finds the arena's buffers through it.
  it("marks its prototype and every batch's geometry for itself, over the prototype's attributes", () => {
    const arena: StaticArena = createArena();
    const geometry: BufferGeometry = arena.createGeometry();

    expect(arena.prototype.hasAttribute(`${EVertexAttribute.CLUSTER_ARENA}${arena.id}`)).toBe(true);
    expect(Object.keys(geometry.attributes)).toEqual(Object.keys(arena.prototype.attributes));
    expect(geometry.getAttribute("position")).toBe(arena.prototype.getAttribute("position"));
    expect(toClusterSource(geometry)).toBe(arena);
    expect(createArena().id).not.toBe(arena.id);
  });
});
