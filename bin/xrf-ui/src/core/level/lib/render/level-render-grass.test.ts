import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, IRendererGrass } from "@xrf/renderer";

import { DetailsModel, VisualSection } from "@/core/ipc/types/xrf-visual";
import { toLevelRendererGrass } from "@/core/level/lib/render/level-render-grass";
import { ILevelGrassDelivery } from "@/core/level/lib/render/level-render-protocol";
import { mockSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";

/** Lays sections out one after another, four-byte aligned, as the packer does. */
class GrassBuffer {
  private readonly parts: Array<Uint8Array> = [];
  private length: number = 0;

  public push(values: Uint32Array | Float32Array | Uint16Array): VisualSection {
    const bytes: Uint8Array = new Uint8Array(values.buffer.slice(0));
    const section: VisualSection = { byteLength: bytes.length, byteOffset: this.length };

    this.parts.push(bytes);
    this.length += Math.ceil(bytes.length / 4) * 4;

    return section;
  }

  public toBuffer(): ArrayBuffer {
    const buffer: Uint8Array = new Uint8Array(this.length);
    let at: number = 0;

    for (const part of this.parts) {
      buffer.set(part, at);
      at += Math.ceil(part.length / 4) * 4;
    }

    return buffer.buffer as ArrayBuffer;
  }
}

function mockGrass(): ILevelGrassDelivery {
  const buffer: GrassBuffer = new GrassBuffer();
  const model: DetailsModel = {
    height: 2,
    indexCount: 3,
    indices: buffer.push(new Uint16Array([0, 2, 1])),
    isWaving: true,
    maxScale: 1.5,
    minScale: 0.5,
    positions: buffer.push(new Float32Array([0, 0, 0, 1, 0, 0, 0, 2, 0])),
    radius: 1.2,
    shader: "details\\blend",
    texture: "detail\\grass",
    uvs: buffer.push(new Float32Array([0, 1, 1, 1, 0.5, 0])),
    vertexCount: 3,
  };
  const grid: VisualSection = buffer.push(new Uint32Array([1, 0]));
  const slots: VisualSection = buffer.push(new Uint32Array([1, 2, 3, 4, 0, 1, 7, 8]));
  const bins: VisualSection = buffer.push(new Uint32Array([0]));
  const triangles: VisualSection = buffer.push(new Float32Array([0, 1, 0, 2, 1, 0, 0, 1, 2]));

  return {
    buffer: buffer.toBuffer(),
    description: {
      details: {
        binLength: 1,
        bins,
        bufferLength: 0,
        grid,
        models: [model],
        offsetX: 3,
        offsetZ: -2,
        sizeX: 2,
        sizeZ: 1,
        slotCount: 1,
        slots,
        triangleCount: 1,
        triangles,
      },
      surfaces: [mockSurfaceDescriptor({ shader: "details\\blend", textures: ["detail\\grass"] })],
      textures: [{ logicalPath: "textures\\detail\\grass.dds", reference: "detail\\grass" }],
    },
  };
}

describe("toLevelRendererGrass", () => {
  it("reads the grid, slots, bins and triangles out of the pack, and each model with its texture cut out", () => {
    const grass: IRendererGrass = toLevelRendererGrass(mockGrass());

    expect(Array.from(grass.grid)).toEqual([1, 0]);
    expect(Array.from(grass.slots)).toEqual([1, 2, 3, 4, 0, 1, 7, 8]);
    expect(Array.from(grass.bins)).toEqual([0]);
    expect(Array.from(grass.triangles)).toEqual([0, 1, 0, 2, 1, 0, 0, 1, 2]);
    expect([grass.sizeX, grass.sizeZ, grass.offsetX, grass.offsetZ]).toEqual([2, 1, 3, -2]);

    const [model] = grass.models;

    expect(Array.from(model.indices)).toEqual([0, 2, 1]);
    expect(Array.from(model.positions)).toEqual([0, 0, 0, 1, 0, 0, 0, 2, 0]);
    expect(Array.from(model.uvs)).toEqual([0, 1, 1, 1, 0.5, 0]);
    expect(model.surface.draw).toBe(ERendererDraw.CUT_OUT);
    expect(model.surface.textures.base).toBe("detail\\grass");
    expect([model.isWaving, model.minScale, model.maxScale, model.height, model.radius]).toEqual([
      true,
      0.5,
      1.5,
      2,
      1.2,
    ]);
  });

  // The renderer takes what it is handed, so the loader's own pack must survive for a renderer started later.
  it("reads a copy, leaving the loader's pack its own", () => {
    const delivery: ILevelGrassDelivery = mockGrass();
    const grass: IRendererGrass = toLevelRendererGrass(delivery);

    expect(grass.grid.buffer).not.toBe(delivery.buffer);
  });
});
