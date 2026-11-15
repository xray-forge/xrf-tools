import { describe, expect, it } from "@jest/globals";
import { SceneDrawDescription, SceneHelpersDescription } from "@xrf/ipc/types/xrf-app-levels";
import {
  ERenderDebugView,
  ERenderSurfaceColor,
  RenderHelperOptions,
  RenderViewOptions,
} from "@xrf/ipc/types/xrf-renderer";

import { mockRenderFeatures } from "@/core/render/fixtures/render.mocks";
import { toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { NO_NATIVE_HELPERS } from "@/core/render/lib/native/contract/native-view-options";
import { NATIVE_RENDER_OUTPUT } from "@/core/render/lib/settings/device/render-resolution";
import {
  MOCK_NO_SCENE_HELPERS,
  mockSceneDrawDescription,
  mockSceneHelpersDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import {
  DEFAULT_LEVEL_EDITOR_LIGHTING,
  LEVEL_EDITOR_WORLD_TOGGLES,
  toLevelEditorSceneBounds,
  toLevelEditorSceneSphere,
  toLevelEditorViewOptions,
} from "@/tools/level-editor/lib/render/level-editor-render";

describe("toLevelEditorViewOptions", () => {
  it("draws the scene textured under the editor's light, with no sky, fog or weather, its helpers as asked", () => {
    const helpers: RenderHelperOptions = { ...NO_NATIVE_HELPERS, isShown: true, kinds: 0b1010 };
    const options: RenderViewOptions = toLevelEditorViewOptions({
      features: mockRenderFeatures(),
      helpers,
      lighting: DEFAULT_LEVEL_EDITOR_LIGHTING,
      output: NATIVE_RENDER_OUTPUT,
      surfaces: { isBumped: true, isWireframe: false },
    });

    expect(options.helpers).toEqual(helpers);

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
        helpers: NO_NATIVE_HELPERS,
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

describe("toLevelEditorSceneBounds", () => {
  const helpers: SceneHelpersDescription = mockSceneHelpersDescription();

  it("frames a scene by its drawn objects where it draws any", () => {
    expect(toLevelEditorSceneBounds(mockSceneDrawDescription(), helpers)).toEqual(mockSceneDrawDescription().bounds);
    expect(toLevelEditorSceneBounds(mockSceneDrawDescription(), null)).toEqual(mockSceneDrawDescription().bounds);
  });

  it("frames a scene drawing no object by its helpers once they are read", () => {
    const undrawn: SceneDrawDescription = { ...mockSceneDrawDescription(), bounds: null };

    expect(toLevelEditorSceneBounds(undrawn, helpers)).toEqual(helpers.bounds);
    expect(toLevelEditorSceneBounds(undrawn, null)).toBeNull();
    expect(toLevelEditorSceneBounds(undrawn, MOCK_NO_SCENE_HELPERS)).toBeNull();
  });

  it("frames nothing before the drawing is read", () => {
    expect(toLevelEditorSceneBounds(null, helpers)).toBeNull();
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
