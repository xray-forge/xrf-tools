import { describe, expect, it } from "@jest/globals";
import { ERenderDebugView, ERenderSurfaceColor, RenderViewOptions } from "@xrf/ipc/types/xrf-renderer";

import { mockRenderFeatures } from "@/core/render/fixtures/render.mocks";
import { toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { NATIVE_RENDER_OUTPUT } from "@/core/render/lib/settings/device/render-resolution";
import {
  DEFAULT_LEVEL_EDITOR_LIGHTING,
  LEVEL_EDITOR_WORLD_TOGGLES,
  toLevelEditorSceneSphere,
  toLevelEditorViewOptions,
} from "@/tools/level-editor/lib/render/level-editor-render";

describe("toLevelEditorViewOptions", () => {
  it("draws the scene textured under the editor's light, with no sky, fog or weather", () => {
    const options: RenderViewOptions = toLevelEditorViewOptions({
      features: mockRenderFeatures(),
      lighting: DEFAULT_LEVEL_EDITOR_LIGHTING,
      output: NATIVE_RENDER_OUTPUT,
      surfaces: { isBumped: true, isWireframe: false },
    });

    expect(options.asset.lighting).toEqual(toNativeAssetLighting(DEFAULT_LEVEL_EDITOR_LIGHTING));
    expect(options.asset.backdrop).toBeNull();
    expect(options.mode).toMatchObject({
      debugView: ERenderDebugView.FINAL,
      isBumped: true,
      isLit: true,
      isWireframe: false,
      surfaceColor: ERenderSurfaceColor.TEXTURED,
    });
    expect(options.show).toMatchObject({ isClouded: false, isFogged: false, isSkyVisible: false });
  });

  it("draws the scene as wires where its surfaces ask", () => {
    expect(
      toLevelEditorViewOptions({
        features: mockRenderFeatures(),
        lighting: DEFAULT_LEVEL_EDITOR_LIGHTING,
        output: NATIVE_RENDER_OUTPUT,
        surfaces: { isBumped: false, isWireframe: true },
      }).mode
    ).toMatchObject({ isBumped: false, isWireframe: true });
  });
});

describe("LEVEL_EDITOR_WORLD_TOGGLES", () => {
  // Every scene object stands in the props group, which nothing hides.
  it("plays the scene's objects alone, silent", () => {
    expect(LEVEL_EDITOR_WORLD_TOGGLES).toMatchObject({
      hiddenSpawnObjects: [],
      isRainy: false,
      isSpawnedItems: false,
      isSpawnedProps: true,
      isSpawnedWeapons: false,
      sound: { isHeard: false },
    });
  });
});

describe("toLevelEditorSceneSphere", () => {
  it("frames by the sphere the bounds state", () => {
    expect(
      toLevelEditorSceneSphere({
        boundingBox: { max: { x: 1, y: 1, z: 1 }, min: { x: -1, y: -1, z: -1 } },
        boundingSphere: { center: { x: 10, y: 0, z: -20 }, radius: 5 },
      })
    ).toEqual({ center: { x: 10, y: 0, z: -20 }, radius: 5 });
  });
});
