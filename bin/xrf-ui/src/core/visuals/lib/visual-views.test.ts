import { describe, expect, it } from "@jest/globals";

import { VisualDescription } from "@/core/bindings/types/xrf-visual";
import {
  countVisualTriangles,
  createVisualCameraFit,
  createVisualViews,
  getVisualSubmeshLevel,
  IVisualModelViews,
} from "@/core/visuals/lib/visual-views";
import {
  mockPackedSubmesh,
  mockSkippedSubmesh,
  mockVisualBounds,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";

describe("visual views", () => {
  it("builds typed array views over the packed sections", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer)],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

    expect(views.submeshes).toHaveLength(1);
    expect(Array.from(views.submeshes[0].positions)).toEqual([0, 0, 0, 1, 0, 0, 0, 1, 0]);
    expect(Array.from(views.submeshes[0].uvs)).toEqual([0, 0, 1, 0, 0, 1]);
    expect(Array.from(views.submeshes[0].indices)).toEqual([0, 1, 2]);
  });

  it("rejects a buffer whose length disagrees with its description", () => {
    // The pair is fetched in two calls, so a mismatch means they came from different reads. Building
    // views anyway would render whatever the offsets happened to land on.
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer)],
      bufferLength: buffer.byteLength + 4,
    });

    expect(() => createVisualViews(description, buffer.toArrayBuffer())).toThrow(/came from different reads/);
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

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

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

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

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

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

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

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

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

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

    expect(views.submeshes.map((it) => it.label)).toEqual(["wpn\\wpn_ak74", "submesh 1"]);
  });

  it("packs several submeshes into one buffer without overlapping", () => {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description: VisualDescription = mockVisualDescription({
      submeshes: [mockPackedSubmesh(buffer, { index: 0 }), mockPackedSubmesh(buffer, { index: 1 })],
      bufferLength: buffer.byteLength,
    });

    const views: IVisualModelViews = createVisualViews(description, buffer.toArrayBuffer());

    expect(views.submeshes).toHaveLength(2);
    expect(views.submeshes[1].positions.byteOffset).toBeGreaterThan(views.submeshes[0].indices.byteOffset);
    expect(views.vertexCount).toBe(6);
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
