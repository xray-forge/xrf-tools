import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind, SceneDrawDescription, SceneHelpersDescription } from "@xrf/ipc/types/xrf-app-levels";
import {
  ERenderDebugView,
  ERenderSurfaceColor,
  RenderHelperOptions,
  RenderViewOptions,
} from "@xrf/ipc/types/xrf-renderer";
import { WorldToggles } from "@xrf/ipc/types/xrf-world";

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
  DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS,
  ILevelEditorHelperOptions,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import {
  DEFAULT_LEVEL_EDITOR_LIGHTING,
  ILevelEditorWorldTogglesInputs,
  toLevelEditorSceneBounds,
  toLevelEditorSceneSphere,
  toLevelEditorViewOptions,
  toLevelEditorWorldToggles,
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

describe("toLevelEditorWorldToggles", () => {
  const none: ILevelEditorHelperOptions = { isShown: false, tools: {} };

  /** The toggles of a scene whose Show action keeps its defaults but for `inputs`. */
  function toToggles(inputs: Partial<ILevelEditorWorldTogglesInputs>): WorldToggles {
    return toLevelEditorWorldToggles({
      hidden: [],
      helpers: DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS,
      isCollisionShown: false,
      isEditorOnlyShown: false,
      ...inputs,
    });
  }

  // Every scene object stands in the props group, which nothing hides; every spawn point's model in the items group.
  it("plays the scene's objects and its spawn points' models alone, silent", () => {
    expect(toToggles({})).toMatchObject({
      hiddenSpawnObjects: [],
      isRainy: false,
      isSpawnedItems: true,
      isSpawnedLamps: false,
      isSpawnedProps: true,
      isSpawnedWeapons: false,
      sound: { isHeard: false },
    });
  });

  // The surfaces the game never draws stand in the weapons group, which nothing else of a scene stands in.
  it("draws the surfaces the game never draws only where asked, whatever the helpers", () => {
    expect(toToggles({ isCollisionShown: true })).toMatchObject({
      isSpawnedLamps: false,
      isSpawnedProps: true,
      isSpawnedWeapons: true,
    });
    expect(toToggles({ helpers: none, isCollisionShown: true }).isSpawnedWeapons).toBe(true);
  });

  // The surfaces dressed in the SDK's marker textures stand in the lamps group, which nothing else of a scene stands in.
  it("draws the surfaces dressed in marker textures only where asked, whatever the helpers", () => {
    expect(toToggles({ isEditorOnlyShown: true })).toMatchObject({
      isSpawnedLamps: true,
      isSpawnedProps: true,
      isSpawnedWeapons: false,
    });
    expect(toToggles({ helpers: none, isEditorOnlyShown: true }).isSpawnedLamps).toBe(true);
  });

  it("switches the spawn points' models with their helpers' row and the master switch over it", () => {
    expect(toToggles({ helpers: { isShown: true, tools: { [ESceneToolKind.SPAWN]: false } } }).isSpawnedItems).toBe(
      false
    );
    expect(toToggles({ helpers: none })).toMatchObject({ isSpawnedItems: false, isSpawnedProps: true });
    expect(toToggles({ helpers: { isShown: true, tools: { [ESceneToolKind.WAY]: false } } }).isSpawnedItems).toBe(true);
  });

  it("hides the objects hidden one by one by their models' render ids, every kind of surface, ascending", () => {
    expect(
      toToggles({
        hidden: [
          { index: 7, tool: ESceneToolKind.SCENE_OBJECT },
          { index: 2, tool: ESceneToolKind.SPAWN },
          { index: 3, tool: ESceneToolKind.SCENE_OBJECT },
        ],
      }).hiddenSpawnObjects
    ).toEqual([
      0x0200_0003, 0x0200_0007, 0x0600_0002, 0x4200_0003, 0x4200_0007, 0x4600_0002, 0x8200_0003, 0x8200_0007,
      0x8600_0002,
    ]);
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
  it("frames by the sphere stated, its centre alone where it states no radius", () => {
    expect(toLevelEditorSceneSphere({ center: { x: 10, y: 0, z: -20 }, radius: 5 })).toEqual({
      center: { x: 10, y: 0, z: -20 },
      radius: 5,
    });
    expect(toLevelEditorSceneSphere({ center: { x: 1, y: 2, z: 3 }, radius: null }).radius).toBe(0);
  });
});
