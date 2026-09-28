import { describe, expect, it, jest } from "@jest/globals";
import { ERendererDraw, ERendererTextureEncoding, IRendererGeometry, IRendererObject } from "@xrf/renderer";
import { mockDdsFile } from "@xrf/renderer/fixtures";

import { SectorDescription } from "@/core/ipc/types/xrf-visual";
import { LevelRenderContent, TLevelRenderSink } from "@/core/level/lib/render/level-render-content";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { ILevelSectorDelivery, ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ILevelSpawnModelsDelivery } from "@/core/level/lib/render/level-render-spawn";
import { toLevelSurfaceColor } from "@/core/level/lib/render/level-render-surface";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { DEFAULT_LEVEL_SURFACE_OPTIONS } from "@/core/level/lib/surface/level-surface-options";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import {
  mockPackedSubmesh,
  mockSurfaceDescriptor,
  MockVisualBuffer,
  mockVisualDescription,
  mockVisualTransform,
} from "@/fixtures/mocks/visual.mocks";

type TMockSink = { [K in keyof TLevelRenderSink]: jest.Mock<TLevelRenderSink[K]> };

function mockSink(): TMockSink {
  return {
    putGeometry: jest.fn(),
    putGrass: jest.fn(),
    putLights: jest.fn(),
    putImpostors: jest.fn(),
    putObject: jest.fn(),
    putSurface: jest.fn(),
    putTexture: jest.fn(),
    releaseGeometry: jest.fn(),
    releaseGrass: jest.fn(),
    releaseLights: jest.fn(),
    releaseImpostors: jest.fn(),
    releaseObject: jest.fn(),
    releaseSurface: jest.fn(),
    releaseTexture: jest.fn(),
  };
}

function mockDelivery(sector: number = 4): ILevelSectorDelivery {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const instance = mockSectorInstanceGroup(buffer, [10, -10], {
    surface: mockSectorSurface({ shaderId: 2, textureName: "tree" }),
  });
  const description: SectorDescription = mockSectorDescription(buffer, {
    instances: [instance],
    sections: [mockSectorSection({ surface: mockSectorSurface({ hemi: "lmap#1_2", shaderId: 1 }) })],
    sector,
  });

  return { buffer: buffer.toArrayBuffer(), description: { ...description, bufferLength: buffer.byteLength }, sector };
}

function mockTexture(reference: string, overrides: Partial<ILevelTextureDelivery> = {}): ILevelTextureDelivery {
  return {
    bytes: mockDdsFile({ height: 8, mipmapCount: 4, width: 8 }),
    isDecoded: false,
    reason: null,
    reference,
    size: { height: 8, levels: 4, width: 8 },
    ...overrides,
  };
}

/** One lamp model of one submesh, standing in one place. */
function mockSpawnModels(): ILevelSpawnModelsDelivery {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockVisualDescription({
    bufferLength: 0,
    submeshes: [mockPackedSubmesh(buffer, { index: 0, textureName: "lamp" })],
  });

  return {
    buffers: new Map([["lamp", buffer.toArrayBuffer()]]),
    description: {
      models: [
        {
          description: { ...description, bufferLength: buffer.byteLength },
          name: "lamp",
          rest: null,
          surfaces: [mockSurfaceDescriptor()],
          textures: [],
        },
      ],
      placements: [
        { model: 0, name: "lamp_0", section: "physic_object", transform: mockVisualTransform({ x: 1, y: 2, z: 3 }) },
      ],
    },
  };
}

function mockContent(): { content: LevelRenderContent; sink: TMockSink } {
  const sink: TMockSink = mockSink();
  const content: LevelRenderContent = new LevelRenderContent(sink);

  content.open([mockSurfaceDescriptor(), mockSurfaceDescriptor(), mockSurfaceDescriptor()]);

  return { content, sink };
}

describe("LevelRenderContent", () => {
  it("puts a sector's baked geometry and each mesh it stands, and one surface per entry they draw", () => {
    const { content, sink } = mockContent();

    content.deliver({ delivered: [mockDelivery()], released: [] });

    expect(sink.putSurface.mock.calls.map(([key]) => key)).toEqual(["surface:1", "surface:2"]);
    expect(sink.putGeometry.mock.calls.map(([key]) => key)).toEqual(["sector:4", "sector:4:instance:0"]);

    const [, sectorGeometry] = sink.putGeometry.mock.calls[0] as [string, IRendererGeometry];
    const [, instanced] = sink.putObject.mock.calls[1] as [string, IRendererObject];

    expect(sectorGeometry.groups).toEqual([{ count: 3, slot: 0, start: 0 }]);
    expect(instanced.surfaces).toEqual(["surface:2"]);
    expect(instanced.instances?.transforms).toHaveLength(32);
    expect(instanced.instances?.hemi).toHaveLength(4);
  });

  // A surface is its shader table entry, shared by every sector drawing it: putting it per sector would build its
  // material once per sector.
  it("puts an entry once however many sectors draw it", () => {
    const { content, sink } = mockContent();

    content.deliver({ delivered: [mockDelivery(4), mockDelivery(5)], released: [] });

    expect(sink.putSurface).toHaveBeenCalledTimes(2);
  });

  it("releases what a sector put when it goes, and holds the rest", () => {
    const { content, sink } = mockContent();

    content.deliver({ delivered: [mockDelivery(4), mockDelivery(5)], released: [] });
    content.deliver({ delivered: [], released: [4] });

    expect(sink.releaseObject.mock.calls.map(([key]) => key)).toEqual(["sector:4", "sector:4:instance:0"]);
    expect(sink.releaseGeometry.mock.calls.map(([key]) => key)).toEqual(["sector:4", "sector:4:instance:0"]);
    expect(content.held().sectors).toBe(1);
  });

  // The bytes move to the renderer once the task ends, so what the panel reads is counted as the sector arrives.
  it("keeps what each entry draws after the sector's bytes have gone", () => {
    const { content } = mockContent();
    const delivery: ILevelSectorDelivery = mockDelivery();

    content.deliver({ delivered: [delivery], released: [] });
    structuredClone(delivery.buffer, { transfer: [delivery.buffer] });

    expect(content.measure().get(1)).toMatchObject({ drawables: 1, triangles: 1 });
    expect(content.measure().get(2)).toMatchObject({ triangles: 2 });
  });

  it("dresses an entry with its base, its lightmap, and its colour only while untextured", () => {
    const { content, sink } = mockContent();

    content.deliver({ delivered: [mockDelivery()], released: [] });

    expect(sink.putSurface.mock.calls[0][1]).toMatchObject({
      draw: ERendererDraw.OPAQUE,
      textures: { base: "stone", hemi: "lmap#1_2" },
    });
    expect(sink.putSurface.mock.calls[0][1].color).toBeUndefined();

    content.setOptions({ ...DEFAULT_LEVEL_SURFACE_OPTIONS, isTextured: false });

    expect(sink.putSurface).toHaveBeenCalledTimes(4);
    expect(sink.putSurface.mock.calls[2][1].textures.base).toBeUndefined();
    expect(sink.putSurface.mock.calls[2][1].color).toHaveLength(3);
  });

  it("uploads each file as it arrives and says what it came to", () => {
    const { content, sink } = mockContent();

    content.supply({ delivered: [mockTexture("stone")], retained: null });

    expect(sink.putTexture.mock.calls[0][1].encoding).toBe(ERendererTextureEncoding.DDS);
    expect(content.describeTextures().dressing.get("stone")).toMatchObject({
      state: ELevelSurfaceDressing.UPLOADED,
      upload: "8×8 · 4 levels",
    });
  });

  // A surface whose file could not be read is drawn from a checker, which reads exactly like a blending fault unless
  // something says so.
  it("stands a checker in for a file that could not be read, and says why", () => {
    const { content, sink } = mockContent();

    content.supply({ delivered: [mockTexture("stone", { reason: "not in any root" })], retained: null });

    expect(sink.putTexture.mock.calls[0][1]).toMatchObject({
      encoding: ERendererTextureEncoding.RGBA,
      isNearest: true,
    });
    expect(content.describeTextures().problems).toEqual([{ reason: "not in any root", reference: "stone" }]);
  });

  it("stands a checker in for a file the renderer refused", () => {
    const { content, sink } = mockContent();

    content.supply({ delivered: [mockTexture("stone")], retained: null });
    content.refuse("stone", { detail: "BC9", reason: "unsupportedFourCc" as never });

    expect(sink.putTexture.mock.calls[1][1].encoding).toBe(ERendererTextureEncoding.RGBA);
    expect(content.describeTextures().dressing.get("stone")?.state).toBe(ELevelSurfaceDressing.STOOD_IN);
  });

  it("lets go of what nothing resident names, and of everything when the set goes", () => {
    const { content, sink } = mockContent();

    content.supply({ delivered: [mockTexture("stone"), mockTexture("tree")], retained: null });
    content.supply({ delivered: [], retained: new Set(["tree"]) });

    expect(sink.releaseTexture.mock.calls.map(([key]) => key)).toEqual(["stone"]);

    content.supply({ delivered: [], retained: null });

    expect(sink.releaseTexture.mock.calls.map(([key]) => key)).toEqual(["stone", "tree"]);
    expect(content.describeTextures().uploaded).toBe(0);
  });

  it("lets the last level go whole when another opens", () => {
    const { content, sink } = mockContent();

    content.deliver({ delivered: [mockDelivery()], released: [] });
    content.supply({ delivered: [mockTexture("stone")], retained: null });
    content.open([]);

    expect(sink.releaseObject).toHaveBeenCalledTimes(2);
    expect(sink.releaseSurface.mock.calls.map(([key]) => key)).toEqual([
      LEVEL_RENDER_KEYS.surface(1),
      LEVEL_RENDER_KEYS.surface(2),
    ]);
    expect(sink.releaseTexture).toHaveBeenCalledWith("stone");
    expect(content.held()).toEqual({ bytes: 0, sectors: 0 });
  });

  it("dresses spawned models again on a texture toggle without putting their geometry again", () => {
    const { content, sink } = mockContent();
    const key: string = LEVEL_RENDER_KEYS.spawn(0, 0);

    content.stand(mockSpawnModels());

    expect(sink.putGeometry.mock.calls.map(([it]) => it)).toEqual([key]);
    expect(sink.putSurface.mock.calls.at(-1)?.[1].textures?.base).toBe("lamp");

    sink.putGeometry.mockClear();
    content.setOptions({ ...DEFAULT_LEVEL_SURFACE_OPTIONS, isTextured: false });

    expect(sink.putGeometry).not.toHaveBeenCalled();
    // Untextured, a model takes a colour of its own as a sector's surface does, rather than drawing white.
    expect(sink.putSurface.mock.calls.at(-1)).toEqual([
      key,
      expect.objectContaining({ color: toLevelSurfaceColor(0), textures: { base: undefined } }),
    ]);
  });

  // A renderer started after the models were read is handed them before the level's own reaction opens it.
  it("keeps spawned models held when a level opens, which the loader alone lets go", () => {
    const { content, sink } = mockContent();

    content.stand(mockSpawnModels());
    content.open([]);

    expect(sink.releaseObject).not.toHaveBeenCalledWith(LEVEL_RENDER_KEYS.spawn(0, 0));

    content.stand(null);

    expect(sink.releaseObject).toHaveBeenCalledWith(LEVEL_RENDER_KEYS.spawn(0, 0));
  });
});
