import { describe, expect, it } from "@jest/globals";
import { DataTexture, NearestFilter, RepeatWrapping } from "three";

import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG, IVisualPreviewSceneConfig } from "./scene-config";
import { createCheckerTexture } from "./VisualPreviewScene.utils";

/** A board small enough to assert every pixel of. */
const TINY_BOARD: IVisualPreviewSceneConfig = {
  ...DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
  checkerSize: 2,
  checkerRepeat: 3,
};

describe("createCheckerTexture", () => {
  it("alternates opaque squares across the board", () => {
    const texture: DataTexture = createCheckerTexture(TINY_BOARD);
    const data: Uint8Array = texture.image.data as Uint8Array;

    // Two by two, so the diagonal is light and the off-diagonal dark, and every pixel is fully opaque.
    expect(Array.from(data)).toEqual([
      0xff, 0xff, 0xff, 0xff, 0x40, 0x40, 0x40, 0xff, 0x40, 0x40, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff,
    ]);
  });

  it("takes its size and repeat from the configuration", () => {
    const texture: DataTexture = createCheckerTexture(TINY_BOARD);

    expect([texture.image.width, texture.image.height]).toEqual([2, 2]);
    expect([texture.repeat.x, texture.repeat.y]).toEqual([3, 3]);
  });

  it("tiles without smoothing, so the squares stay squares", () => {
    // A filtered checkerboard blurs into grey at a distance, which is useless for judging uv layout.
    const texture: DataTexture = createCheckerTexture(DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG);

    expect(texture.wrapS).toBe(RepeatWrapping);
    expect(texture.wrapT).toBe(RepeatWrapping);
    expect(texture.magFilter).toBe(NearestFilter);
    expect(texture.minFilter).toBe(NearestFilter);
    // `needsUpdate` is write only in three; setting it bumps the version, which is the readable effect.
    expect(texture.version).toBeGreaterThan(0);
  });
});
