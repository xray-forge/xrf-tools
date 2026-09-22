import { describe, expect, it } from "@jest/globals";

import {
  ETextureSurfaceRequest,
  listTextureSurfaceTransfers,
} from "@/core/textures/lib/render/texture-surface-messages";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/scene/texture-lighting";
import { EMPTY_TEXTURE_SURFACE, ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";

function mockFiles(): ITextureSurfaceFiles {
  const file = { bytes: new ArrayBuffer(64), height: 8, isDecoded: false, width: 8 };

  return { aspect: 1, base: file, bump: { bump: file, companion: file } };
}

describe("listTextureSurfaceTransfers", () => {
  // An OffscreenCanvas cannot be cloned at all: posting one without transferring it throws, and the whole
  // surface comes up as a render error instead of a picture.
  it("moves the canvas, because a canvas cannot be copied", () => {
    const canvas = {} as OffscreenCanvas;

    expect(
      listTextureSurfaceTransfers({
        canvas,
        height: 1,
        kind: ETextureSurfaceRequest.START,
        pixelRatio: 1,
        width: 1,
      })
    ).toEqual([canvas]);
  });

  // The channels panel draws the pair's planes from the same bytes on this thread: moving them would leave it
  // holding a detached buffer, and a texture file is small enough that copying it costs nothing worth having.
  it("copies the texture files rather than taking them away", () => {
    const files: ITextureSurfaceFiles = mockFiles();

    expect(listTextureSurfaceTransfers({ files, kind: ETextureSurfaceRequest.TEXTURES })).toEqual([]);
    expect(files.base?.bytes.byteLength).toBe(64);
  });

  it("moves nothing for a message that carries nothing", () => {
    expect(listTextureSurfaceTransfers({ kind: ETextureSurfaceRequest.DISPOSE })).toEqual([]);
    expect(listTextureSurfaceTransfers({ kind: ETextureSurfaceRequest.RESET })).toEqual([]);
    expect(listTextureSurfaceTransfers({ kind: ETextureSurfaceRequest.DOLLY, step: 2 })).toEqual([]);
    expect(
      listTextureSurfaceTransfers({ height: 2, kind: ETextureSurfaceRequest.RESIZE, pixelRatio: 1, width: 3 })
    ).toEqual([]);
    expect(
      listTextureSurfaceTransfers({ kind: ETextureSurfaceRequest.LIGHTING, lighting: DEFAULT_TEXTURE_LIGHTING })
    ).toEqual([]);
    expect(
      listTextureSurfaceTransfers({ files: EMPTY_TEXTURE_SURFACE, kind: ETextureSurfaceRequest.TEXTURES })
    ).toEqual([]);
  });
});
