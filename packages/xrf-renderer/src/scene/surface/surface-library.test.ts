import { describe, expect, it } from "@jest/globals";
import { WebGPURenderer } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockDdsFile } from "#/dds/dds-fixtures";
import { SurfaceLibrary } from "#/scene/surface/surface-library";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** A renderer that uploads nothing, which is all the queue asks of it here. */
const RENDERER: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

const WALL: IRendererSurface = { draw: ERendererDraw.OPAQUE, textures: { base: "brick", hemi: "lmap#1" } };

describe("SurfaceLibrary", () => {
  it("builds a surface again whose batch moved, and retires the shared material it left", () => {
    const replaced: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {}
    );
    const library: SurfaceLibrary = new SurfaceLibrary(
      textures,
      new RendererUniforms(),
      (key: string) => replaced.push(key),
      () => {}
    );

    function upload(key: string, fourCC: string = "DXT5"): void {
      textures.put(key, { bytes: mockDdsFile({ fourCC }), encoding: ERendererTextureEncoding.DDS });
      textures.upload(RENDERER, Infinity);
    }

    upload("brick");
    upload("lmap#1");
    library.put("wall", WALL);
    replaced.length = 0;

    expect(library.hasRetired).toBe(false);

    // Of another class: the wall moves to another shared material, and the one it drew by goes idle.
    upload("brick", "DXT1");
    library.rebind("brick");

    expect(replaced).toEqual(["wall"]);
    expect(library.hasRetired).toBe(true);

    library.retire(new Set(), () => false);

    expect(library.hasRetired).toBe(false);
  });
});
