import { describe, expect, it } from "@jest/globals";

import { VisualDescription } from "@/core/ipc/types/xrf-visual";
import { ERenderDraw } from "@/core/render/lib/surface/render-draw";
import { OPAQUE_RENDER_SURFACE_DRAW } from "@/core/render/lib/surface/render-surface-draw";
import { createVisualSurfaces } from "@/core/visuals/lib/visual-surface";
import {
  countVisualTriangles,
  createVisualCameraFit,
  createVisualViews,
  getVisualSubmeshLevel,
  IVisualModelViews,
} from "@/core/visuals/lib/visual-views";
import {
  mockAlphaSurfaceDescriptor,
  mockPackedSubmesh,
  mockSkippedSubmesh,
  mockVisualBone,
  mockVisualBounds,
  MockVisualBuffer,
  mockVisualDescription,
  mockVisualTransform,
} from "@/fixtures/mocks/visual.mocks";

describe("visual views", () => {
  it("carries the material state each submesh's shader compiles to", () => {
    // With the geometry rather than after it, so a cut-out is never drawn solid on the way in.
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [
        mockPackedSubmesh(buffer, { index: 0, shaderName: "models\\model_aref" }),
        mockPackedSubmesh(buffer, { index: 1, shaderName: "models\\model" }),
      ],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(
      description,
      createVisualSurfaces(description.submeshes, [mockAlphaSurfaceDescriptor()])
    );

    expect(views.submeshes[0].surface.draw).toBe(ERenderDraw.CUT_OUT);
    // A submesh the table has no answer for is opaque, which is what a model opened without a library gets.
    expect(views.submeshes[1].surface).toEqual(OPAQUE_RENDER_SURFACE_DRAW);
  });

  it("keeps every detail level separate from the whole index buffer", () => {
    // A progressive submesh ships every detail level and uploads the whole buffer. Which of them is drawn is a range,
    // so a level the viewer picks costs no second read.
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const submesh = mockPackedSubmesh(
      buffer,
      {},
      {
        indexCount: 12,
        detailLevels: [
          { start: 6, count: 6 },
          { start: 0, count: 12 },
        ],
      }
    );
    const description: VisualDescription = mockVisualDescription({
      submeshes: [submesh],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description);

    expect(views.levelCount).toBe(2);
    expect(views.submeshes[0].levels).toEqual([
      { start: 6, count: 6, triangleCount: 2 },
      { start: 0, count: 12, triangleCount: 4 },
    ]);
    expect(countVisualTriangles(views, 0)).toBe(2);
    expect(countVisualTriangles(views, 1)).toBe(4);
  });

  it("takes the same share of each submesh's chain rather than the same level index", () => {
    // A measured character carries 230 collapse steps on one submesh and 948 on another. A shared index would drive
    // the short chain to its coarsest while the long one was a quarter down, decimating the model unevenly.
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const long = mockPackedSubmesh(
      buffer,
      { index: 0 },
      {
        indexCount: 12,
        detailLevels: [
          { start: 6, count: 6 },
          { start: 3, count: 6 },
          { start: 0, count: 12 },
        ],
      }
    );
    const short = mockPackedSubmesh(buffer, { index: 1 }, { indexCount: 3, detailLevels: [{ start: 0, count: 3 }] });
    const description: VisualDescription = mockVisualDescription({
      submeshes: [long, short],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description);

    // Halfway down a three-entry chain is its middle entry, and a one-entry chain has nowhere to go.
    expect(getVisualSubmeshLevel(views.submeshes[0], 0.5)).toEqual({ start: 3, count: 6, triangleCount: 2 });
    expect(getVisualSubmeshLevel(views.submeshes[1], 0.5)).toEqual({ start: 0, count: 3, triangleCount: 1 });
    expect(countVisualTriangles(views, 1)).toBe(5);
  });

  it("holds detail inside its range whatever it is given", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [
        mockPackedSubmesh(
          buffer,
          {},
          {
            indexCount: 12,
            detailLevels: [
              { start: 6, count: 6 },
              { start: 0, count: 12 },
            ],
          }
        ),
      ],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description);

    expect(getVisualSubmeshLevel(views.submeshes[0], -1).start).toBe(6);
    expect(getVisualSubmeshLevel(views.submeshes[0], 42).start).toBe(0);
  });

  it("leaves out submeshes that packed nothing", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const packed = mockPackedSubmesh(buffer, { index: 0 });
    const description: VisualDescription = mockVisualDescription({
      submeshes: [packed, mockSkippedSubmesh({ index: 1 })],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description);

    expect(views.submeshes.map((it) => it.index)).toEqual([0]);
    expect(views.vertexCount).toBe(3);
  });

  it("labels a submesh by its texture, falling back to its index", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [
        mockPackedSubmesh(buffer, { index: 0, textureName: "wpn\\wpn_ak74" }),
        mockPackedSubmesh(buffer, { index: 1, textureName: null }),
      ],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description);

    expect(views.submeshes.map((it) => it.label)).toEqual(["wpn\\wpn_ak74", "submesh 1"]);
  });

  it("counts the vertices of every submesh it keeps", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer, { index: 0 }), mockPackedSubmesh(buffer, { index: 1 })],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description);

    expect(views.submeshes).toHaveLength(2);
    expect(views.vertexCount).toBe(6);
  });
});

describe("visual skeleton", () => {
  it("has a skeleton where a placed bone hangs from a placed parent", () => {
    const description: VisualDescription = mockVisualDescription({
      bones: [
        mockVisualBone({ name: "root", bindTransform: mockVisualTransform({ x: 0, y: 0, z: 0 }) }),
        mockVisualBone({
          name: "spine",
          parent: "root",
          parentIndex: 0,
          bindTransform: mockVisualTransform({ x: 0, y: 1, z: 0 }),
        }),
      ],
    });

    expect(createVisualViews(description).hasSkeleton).toBe(true);
  });

  it("has none when the model carries no bind positions", () => {
    // A visual with no IK chunk still lists its hierarchy, so the bones are present and only the positions are not.
    const description: VisualDescription = mockVisualDescription({
      bones: [mockVisualBone({ name: "root" }), mockVisualBone({ name: "spine", parent: "root", parentIndex: 0 })],
    });

    expect(createVisualViews(description).hasSkeleton).toBe(false);
  });

  it("has none where no placed bone reaches a placed parent", () => {
    const description: VisualDescription = mockVisualDescription({
      bones: [
        mockVisualBone({
          name: "orphan",
          parent: "missing",
          parentIndex: null,
          bindTransform: mockVisualTransform({ x: 5, y: 5, z: 5 }),
        }),
      ],
    });

    expect(createVisualViews(description).hasSkeleton).toBe(false);
  });
});

describe("visual camera fit", () => {
  it("frames what the geometry spans rather than what the header claims", () => {
    const fit = createVisualCameraFit(
      mockVisualDescription({
        declaredBounds: mockVisualBounds({ boundingSphere: { center: { x: 9, y: 9, z: 9 }, radius: 99 } }),
        computedBounds: mockVisualBounds({ boundingSphere: { center: { x: 1, y: 2, z: 3 }, radius: 4 } }),
      })
    );

    expect(fit).toEqual({ center: [1, 2, 3], radius: 4 });
  });

  it("falls back to the declared extent when nothing packed", () => {
    const fit = createVisualCameraFit(
      mockVisualDescription({
        declaredBounds: mockVisualBounds({ boundingSphere: { center: { x: 5, y: 0, z: 0 }, radius: 7 } }),
        computedBounds: null,
      })
    );

    expect(fit).toEqual({ center: [5, 0, 0], radius: 7 });
  });

  it("treats an absent coordinate as no value rather than as zero", () => {
    // A rust f32 crosses as `number | null`, and two visuals in the reference trees declare bounds of
    // f32::MAX. Reading null as zero would place the camera at the origin and claim it framed the model.
    const fit = createVisualCameraFit(
      mockVisualDescription({
        computedBounds: mockVisualBounds({ boundingSphere: { center: { x: null, y: 2, z: 3 }, radius: null } }),
      })
    );

    expect(fit.center).toEqual([0, 0, 0]);
    expect(fit.radius).toBe(1);
    expect(Number.isFinite(fit.radius)).toBe(true);
  });

  it("refuses a degenerate radius so a camera still has somewhere to stand", () => {
    const fit = createVisualCameraFit(
      mockVisualDescription({
        computedBounds: mockVisualBounds({ boundingSphere: { center: { x: 0, y: 0, z: 0 }, radius: 0 } }),
      })
    );

    expect(fit.radius).toBe(1);
  });
});
