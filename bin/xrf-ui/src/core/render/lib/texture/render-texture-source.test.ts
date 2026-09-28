import { describe, expect, it } from "@jest/globals";
import { ERendererTextureEncoding, TRendererTextureSource } from "@xrf/renderer";

import { createRenderCheckerSource, toRendererTextureSource } from "@/core/render/lib/texture/render-texture-source";

describe("toRendererTextureSource", () => {
  it("uploads a file as stored, and the backend's picture of one as a png", () => {
    const bytes: ArrayBuffer = new ArrayBuffer(4);

    expect(toRendererTextureSource(bytes, false)).toEqual({ bytes, encoding: ERendererTextureEncoding.DDS });
    expect(toRendererTextureSource(bytes, true)).toEqual({
      bytes,
      encoding: ERendererTextureEncoding.IMAGE,
      type: "image/png",
    });
  });
});

describe("createRenderCheckerSource", () => {
  it("alternates its two colours by square, opaque and sampled nearest", () => {
    const source: TRendererTextureSource = createRenderCheckerSource(
      4,
      [
        [255, 0, 255],
        [16, 16, 16],
      ],
      2
    );

    expect(source).toMatchObject({ encoding: ERendererTextureEncoding.RGBA, height: 4, isNearest: true, width: 4 });

    if (!("bytes" in source)) {
      throw new Error("A checker carries its texels");
    }

    const texels: Uint8Array = new Uint8Array(source.bytes);

    function at(x: number, y: number): Array<number> {
      return Array.from(texels.subarray((y * 4 + x) * 4, (y * 4 + x) * 4 + 4));
    }

    expect(at(0, 0)).toEqual([255, 0, 255, 255]);
    expect(at(1, 1)).toEqual([255, 0, 255, 255]);
    expect(at(2, 0)).toEqual([16, 16, 16, 255]);
    expect(at(2, 2)).toEqual([255, 0, 255, 255]);
  });
});
