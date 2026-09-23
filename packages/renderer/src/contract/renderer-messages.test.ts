import { describe, expect, it } from "@jest/globals";

import { ERendererRequest, listRendererTransfers } from "#/contract/renderer-messages";
import { ERendererDebugView } from "#/contract/renderer-settings";
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
          isBumped: true,
          isLit: true,
          tonemapScale: 1,
        },
      })
    ).toEqual([]);
  });
});
