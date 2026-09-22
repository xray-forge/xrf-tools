import { describe, expect, it } from "@jest/globals";

import {
  ERenderFrame,
  isRenderFrameRequest,
  listRenderFrameTransfers,
} from "@/core/render/lib/worker/render-frame-messages";
import { ERenderInput } from "@/core/render/lib/worker/render-input";

describe("listRenderFrameTransfers", () => {
  // An OffscreenCanvas cannot be cloned at all: posting one without transferring it throws, and the whole
  // viewport comes up as a render error instead of a picture. Every worker-drawn viewport says this message.
  it("moves the canvas, because a canvas cannot be copied", () => {
    const canvas = {} as OffscreenCanvas;

    expect(listRenderFrameTransfers({ canvas, height: 1, kind: ERenderFrame.START, pixelRatio: 1, width: 1 })).toEqual([
      canvas,
    ]);
  });

  it("moves nothing else of the frame", () => {
    expect(listRenderFrameTransfers({ kind: ERenderFrame.DISPOSE })).toEqual([]);
    expect(listRenderFrameTransfers({ height: 2, kind: ERenderFrame.RESIZE, pixelRatio: 1, width: 3 })).toEqual([]);
  });

  // A viewport's own messages carry what the panels on the page read as well, and are copies for that reason.
  it("moves nothing of a viewport's own", () => {
    expect(listRenderFrameTransfers({ kind: "textures" })).toEqual([]);
  });
});

describe("isRenderFrameRequest", () => {
  // The two vocabularies share a union, so they must never collide: a viewport answering `start` for itself
  // would answer the frame's as well.
  it("tells the frame's messages from a viewport's own", () => {
    expect(isRenderFrameRequest({ kind: ERenderFrame.START })).toBe(true);
    expect(isRenderFrameRequest({ kind: ERenderFrame.INPUT })).toBe(true);
    expect(isRenderFrameRequest({ kind: "start" })).toBe(false);
    expect(isRenderFrameRequest({ kind: "model" })).toBe(false);
    expect(isRenderFrameRequest({ kind: ERenderInput.POINTER_DOWN })).toBe(false);
  });
});
