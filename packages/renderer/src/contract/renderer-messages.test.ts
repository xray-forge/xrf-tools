import { describe, expect, it } from "@jest/globals";

import { ERendererRequest, listRendererTransfers } from "#/contract/renderer-messages";
import { DEFAULT_RENDERER_LOD_SETTINGS, ERendererDebugView } from "#/contract/renderer-settings";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";

describe("listRendererTransfers", () => {
  it("moves the canvas with the view that shows it", () => {
    const canvas = {} as OffscreenCanvas;

    expect(
      listRendererTransfers({ canvas, height: 1, kind: ERendererRequest.ATTACH_VIEW, pixelRatio: 1, width: 1 })
    ).toEqual([canvas]);
  });

  it("moves a texture's bytes rather than copying them", () => {
    const bytes: ArrayBuffer = new ArrayBuffer(4);

    expect(
      listRendererTransfers({
        key: "a",
        kind: ERendererRequest.PUT_TEXTURE,
        source: { bytes, encoding: ERendererTextureEncoding.DDS },
      })
    ).toEqual([bytes]);
  });

  // A level's sector is one buffer every attribute is a view over; listed twice, the post throws.
  it("moves a buffer several arrays view once, across a batch", () => {
    const buffer: ArrayBuffer = new ArrayBuffer(64);
    const position: Float32Array = new Float32Array(buffer, 0, 9);
    const hemi: Float32Array = new Float32Array(buffer, 36, 3);
    const transforms: Float32Array = new Float32Array(16);

    expect(
      listRendererTransfers({
        kind: ERendererRequest.BATCH,
        requests: [
          { geometry: { groups: [], hemi, position }, key: "a", kind: ERendererRequest.PUT_GEOMETRY },
          { geometry: { groups: [], position }, key: "b", kind: ERendererRequest.PUT_GEOMETRY },
          {
            key: "c",
            kind: ERendererRequest.PUT_OBJECT,
            object: { geometry: "b", instances: { transforms }, surfaces: [] },
          },
        ],
      })
    ).toEqual([buffer, transforms.buffer]);
  });

  it("moves every array a geometry has, and only those", () => {
    const position: Float32Array = new Float32Array(9);
    const index: Uint16Array = new Uint16Array(3);

    expect(
      listRendererTransfers({
        geometry: { groups: [], index, position },
        key: "a",
        kind: ERendererRequest.PUT_GEOMETRY,
      })
    ).toEqual([position.buffer, index.buffer]);
  });

  it("moves nothing with any other message", () => {
    expect(listRendererTransfers({ height: 1, kind: ERendererRequest.RESIZE, pixelRatio: 1, width: 1 })).toEqual([]);
    expect(listRendererTransfers({ kind: ERendererRequest.DISPOSE })).toEqual([]);
    expect(
      listRendererTransfers({
        kind: ERendererRequest.START,
        settings: {
          backdrop: 0,
          debugView: ERendererDebugView.FINAL,
          frameRateLimit: "60",
          hemiStrength: 1,
          isBumped: true,
          lod: DEFAULT_RENDERER_LOD_SETTINGS,
          isLit: true,
          isWireframe: false,
          tonemapScale: 1,
        },
      })
    ).toEqual([]);
  });
});
