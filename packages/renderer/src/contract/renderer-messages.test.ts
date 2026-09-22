import { describe, expect, it } from "@jest/globals";

import { ERendererRequest, listRendererTransfers } from "#/contract/renderer-messages";

describe("listRendererTransfers", () => {
  it("moves the canvas with the start message", () => {
    const canvas = {} as OffscreenCanvas;

    expect(
      listRendererTransfers({
        canvas,
        configuration: { backdrop: 0, frameRateLimit: "60" },
        height: 1,
        kind: ERendererRequest.START,
        pixelRatio: 1,
        width: 1,
      })
    ).toEqual([canvas]);
  });

  it("moves nothing with any other message", () => {
    expect(listRendererTransfers({ height: 1, kind: ERendererRequest.RESIZE, pixelRatio: 1, width: 1 })).toEqual([]);
    expect(listRendererTransfers({ kind: ERendererRequest.DISPOSE })).toEqual([]);
  });
});
