import { describe, expect, it } from "@jest/globals";

import { getEditorHandOverPaths } from "@/tools/level-decompiler/lib/editor-hand-over-paths";

const SCENE: string = "D:\\qa\\labx8\\rawdata\\levels\\labx8.level";

describe("getEditorHandOverPaths", () => {
  // The editor's empty list reads `objects` beside the scene, where the decompile wrote them.
  it("leaves the objects to the editor's default where the decompile wrote them there and placed no library", () => {
    expect(getEditorHandOverPaths(SCENE, null, [])).toEqual({ scene: SCENE, objects: [] });
  });

  // A root named replaces the editor's default, so the decompile's own objects must be named first.
  it("names the objects beside the scene before the libraries it placed objects from", () => {
    expect(getEditorHandOverPaths(SCENE, null, [" C:\\SDK\\rawdata\\objects ", ""])).toEqual({
      scene: SCENE,
      objects: ["D:\\qa\\labx8\\rawdata\\objects", "C:\\SDK\\rawdata\\objects"],
    });
  });

  it("names the objects root the decompile was given before the libraries", () => {
    expect(getEditorHandOverPaths(SCENE, "D:\\qa\\objects", ["C:\\SDK\\rawdata\\objects"]).objects).toEqual([
      "D:\\qa\\objects",
      "C:\\SDK\\rawdata\\objects",
    ]);
    expect(getEditorHandOverPaths(SCENE, "D:\\qa\\objects", []).objects).toEqual(["D:\\qa\\objects"]);
  });
});
