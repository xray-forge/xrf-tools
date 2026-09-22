import { describe, expect, it } from "@jest/globals";

import { EVisualPreviewRequest, listVisualPreviewTransfers } from "@/core/visuals/lib/render/visual-preview-messages";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";

function mockFile(): IVisualTextureFile {
  return { bytes: new ArrayBuffer(64), isAlphaRead: false, isDecoded: false, logicalPath: "textures\\wall" };
}

describe("listVisualPreviewTransfers", () => {
  // An OffscreenCanvas cannot be cloned at all: posting one without transferring it throws, and the whole
  // preview comes up as a render error instead of a picture.
  it("moves the canvas, because a canvas cannot be copied", () => {
    const canvas = {} as OffscreenCanvas;

    expect(
      listVisualPreviewTransfers({ canvas, height: 1, kind: EVisualPreviewRequest.START, pixelRatio: 1, width: 1 })
    ).toEqual([canvas]);
  });

  // The panels on this thread read the same buffers: moving them would leave the materials and bones panels
  // holding detached memory while the model still draws.
  it("copies what the panels also read", () => {
    const file: IVisualTextureFile = mockFile();

    expect(listVisualPreviewTransfers({ file, kind: EVisualPreviewRequest.TEXTURE, submeshIndex: 0 })).toEqual([]);
    expect(file.bytes.byteLength).toBe(64);

    expect(
      listVisualPreviewTransfers({
        files: { bump: file, companion: file },
        kind: EVisualPreviewRequest.BUMP,
        submeshIndex: 0,
      })
    ).toEqual([]);
    expect(
      listVisualPreviewTransfers({
        kind: EVisualPreviewRequest.POSE,
        pose: { floatsPerBone: 12, frame: 0, transforms: new Float32Array(12) },
      })
    ).toEqual([]);
  });

  it("moves nothing for a message that carries nothing", () => {
    expect(listVisualPreviewTransfers({ kind: EVisualPreviewRequest.DISPOSE })).toEqual([]);
    expect(listVisualPreviewTransfers({ kind: EVisualPreviewRequest.RESET })).toEqual([]);
    expect(listVisualPreviewTransfers({ detail: 0.5, kind: EVisualPreviewRequest.DETAIL })).toEqual([]);
    expect(listVisualPreviewTransfers({ kind: EVisualPreviewRequest.MODEL, model: null })).toEqual([]);
  });
});
