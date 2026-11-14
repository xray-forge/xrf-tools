import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind } from "@xrf/ipc/types/xrf-app-levels";

import { getSceneToolLabel, SCENE_TOOL_LABELS } from "@/tools/level-editor/lib/tools/scene-tools";

describe("scene tools", () => {
  it("knows every tool the SDK saves by the name its part files carry, in the SDK's order", () => {
    expect(Object.values(ESceneToolKind)).toEqual([
      "group",
      "glow",
      "scene_object",
      "light",
      "shape",
      "sound_src",
      "spawn",
      "way",
      "sector",
      "portal",
      "sound_env",
      "ps",
      "detail_object",
      "ai_map",
      "wallmark",
      "fog_volume",
    ]);
  });

  it("labels every tool the SDK saves, and names any other by its own name", () => {
    expect(Object.keys(SCENE_TOOL_LABELS)).toHaveLength(16);
    expect(getSceneToolLabel(ESceneToolKind.SOUND_SOURCE, "sound_src")).toBe("Sound sources");
    expect(getSceneToolLabel(null, "custom_tool")).toBe("custom_tool");
  });
});
