import { describe, expect, it } from "@jest/globals";
import { DataTexture, NearestFilter, RepeatWrapping } from "three";

import { createCheckerTexture } from "@/core/render/lib/render-texture";

describe("createCheckerTexture", () => {
  // The surfaces that most need this are the cut-out ones. Anything below their reference would be discarded and the
  // surface would go on saying nothing, which is the state this exists to end.
  it("is fully opaque, so a cut-out surface cannot discard it", () => {
    const data: Uint8Array = createCheckerTexture().image.data as Uint8Array;

    for (let at = 3; at < data.length; at += 4) {
      expect(data[at]).toBe(255);
    }
  });

  it("alternates two colours rather than being flat", () => {
    const texture: DataTexture = createCheckerTexture();
    const data: Uint8Array = texture.image.data as Uint8Array;
    const colours: Set<string> = new Set();

    for (let at = 0; at < data.length; at += 4) {
      colours.add(`${data[at]},${data[at + 1]},${data[at + 2]}`);
    }

    expect(colours.size).toBe(2);
  });

  // Filtered, a checker at distance averages to flat grey - the one thing it must never look like. Tiled, because a
  // surface's uv leaves [0, 1] and a clamped stand-in would smear one square across it.
  it("tiles and stays sharp", () => {
    const texture: DataTexture = createCheckerTexture();

    expect(texture.magFilter).toBe(NearestFilter);
    expect(texture.minFilter).toBe(NearestFilter);
    expect(texture.wrapS).toBe(RepeatWrapping);
    expect(texture.wrapT).toBe(RepeatWrapping);
  });

  // One texture per faulty reference, so disposing a level's textures stays one rule with no exception in it.
  it("hands out its own texture each time, over pixels it only builds once", () => {
    const first: DataTexture = createCheckerTexture();
    const second: DataTexture = createCheckerTexture();

    expect(first).not.toBe(second);
    expect(first.image.data).toBe(second.image.data);
  });
});
