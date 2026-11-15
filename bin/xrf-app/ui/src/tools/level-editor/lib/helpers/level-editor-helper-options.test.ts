import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind, SCENE_HELPER_TOOLS, SceneHelperToolEntry } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";

import { NO_NATIVE_HELPERS } from "@/core/render/lib/native/contract/native-view-options";
import {
  DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS,
  getSceneHelperTool,
  isLevelEditorHelperOn,
  toLevelEditorHelperOptions,
  toLevelEditorHelperRenderOptions,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";

/** Every helped tool's bit. */
const EVERY_KIND: number = SCENE_HELPER_TOOLS.reduce(
  (bits: number, entry: SceneHelperToolEntry) => (bits | (1 << entry.kind)) >>> 0,
  0
);

describe("toLevelEditorHelperOptions", () => {
  // A scene of helpers alone opens showing them.
  it("draws every helped tool by default", () => {
    expect(toLevelEditorHelperOptions(null)).toEqual(DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS);
    expect(DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS.isShown).toBe(true);

    for (const entry of SCENE_HELPER_TOOLS) {
      expect(isLevelEditorHelperOn(DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS, entry.tool)).toBe(entry.isHelpedByDefault);
    }
  });

  it("keeps what was stored, leaving a tool the table does not help and a switch it does not hold to the default", () => {
    expect(
      toLevelEditorHelperOptions({
        isShown: false,
        tools: { [ESceneToolKind.SCENE_OBJECT]: false, [ESceneToolKind.WAY]: false, [ESceneToolKind.SPAWN]: "no" },
      })
    ).toEqual({ isShown: false, tools: { [ESceneToolKind.WAY]: false } });
    expect(toLevelEditorHelperOptions({ isShown: "yes" }).isShown).toBe(true);
  });

  it("draws no helper for a tool drawing none", () => {
    expect(isLevelEditorHelperOn(DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS, ESceneToolKind.SECTOR)).toBe(false);
  });
});

describe("toLevelEditorHelperRenderOptions", () => {
  it("switches each helped tool by its place among the tools, nothing singled out", () => {
    expect(toLevelEditorHelperRenderOptions(DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS)).toEqual({
      ...NO_NATIVE_HELPERS,
      isShown: true,
      kinds: EVERY_KIND,
    });
  });

  it("leaves out a tool switched off and keeps each tool's choice while helpers are off", () => {
    const options: RenderHelperOptions = toLevelEditorHelperRenderOptions({
      isShown: false,
      tools: { [ESceneToolKind.SPAWN]: false },
    });

    expect(options.isShown).toBe(false);
    expect(options.kinds).toBe((EVERY_KIND & ~(1 << (getSceneHelperTool(ESceneToolKind.SPAWN)?.kind ?? 0))) >>> 0);
  });
});
