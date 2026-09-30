import { describe, expect, it } from "@jest/globals";
import { mockDdsFile } from "@xrf/dds/fixtures";
import { Nullable } from "@xrf/types";
import { WebGPURenderer } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SurfaceLibrary } from "#/scene/surface/surface-library";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** A renderer that uploads nothing, which is all the queue asks of it here. */
const RENDERER: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

const WALL: IRendererSurface = { draw: ERendererDraw.OPAQUE, textures: { base: "brick", hemi: "lmap#1" } };

interface ILibraryFixture {
  library: SurfaceLibrary;
  uniforms: RendererUniforms;
  /** Each key the library said was replaced, with what lets go of the view it drew by. */
  replaced: Array<readonly [string, Nullable<() => void>]>;
  upload(key: string, fourCC?: string): void;
}

function createFixture(): ILibraryFixture {
  const replaced: Array<readonly [string, Nullable<() => void>]> = [];
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );
  const uniforms: RendererUniforms = new RendererUniforms();

  return {
    library: new SurfaceLibrary(
      textures,
      uniforms,
      (key: string, release: Nullable<() => void>) => replaced.push([key, release]),
      () => {}
    ),
    replaced,
    uniforms,
    upload: (key: string, fourCC: string = "DXT5"): void => {
      textures.put(key, { bytes: mockDdsFile({ fourCC }), encoding: ERendererTextureEncoding.DDS });
      textures.upload(RENDERER, Infinity);
    },
  };
}

describe("SurfaceLibrary", () => {
  it("builds a surface again whose batch moved, and retires the shared material it left once its view is let go", () => {
    const { library, replaced, upload } = createFixture();

    upload("brick");
    upload("lmap#1");
    library.put("wall", WALL);
    replaced.length = 0;

    expect(library.hasRetired).toBe(false);

    // Of another class: the wall moves to another shared material, and the one it drew by goes idle once let go.
    upload("brick", "DXT1");
    library.rebind("brick");

    const [[key, release]] = replaced;

    expect(key).toBe("wall");
    expect(library.hasRetired).toBe(false);

    release?.();

    expect(library.hasRetired).toBe(true);

    library.retire(new Set(), () => false);

    expect(library.hasRetired).toBe(false);
  });

  // Its static draws read its row until the change rebuilding them applies, which is when the release runs.
  it("hands the release of a released surface's view to the change rebuilding its users", () => {
    const { library, replaced, uniforms, upload } = createFixture();

    upload("brick");
    upload("lmap#1");
    library.put("wall", WALL);

    const row: number = (library.get("wall")?.batched as ISurfaceMaterial).row;

    replaced.length = 0;
    library.release("wall");

    const [[key, release]] = replaced;

    expect(key).toBe("wall");
    expect(uniforms.surfaceTable.allocate()).not.toBe(row);

    release?.();

    expect(uniforms.surfaceTable.allocate()).toBe(row);
  });
});
