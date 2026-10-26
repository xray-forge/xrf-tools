import { describe, expect, it } from "@jest/globals";

import { IEditorPanel } from "@/core/shell/editor-shell";

import { TEXTURES_EDITOR_PANELS } from "./textures-editor-panels";

describe("createTexturesEditorPanels", () => {
  it("registers no tree, whatever the session holds", () => {
    expect(TEXTURES_EDITOR_PANELS.map((panel: IEditorPanel) => panel.id)).toEqual([
      "descriptor",
      "formats",
      "bump",
      "material",
      "files",
      "channels",
    ]);
    expect(TEXTURES_EDITOR_PANELS.some((panel: IEditorPanel) => panel.side === "left")).toBe(false);
  });

  it("opens the descriptor first, because that is what this tool is for", () => {
    // The explorer opens `Material` instead: the same `.thm` read from the other end. Which one is in front is the
    // difference between a tool for understanding a texture and a tool for changing one.
    const open: Array<string> = TEXTURES_EDITOR_PANELS.filter((panel: IEditorPanel) => panel.isOpenByDefault).map(
      (panel: IEditorPanel) => panel.id
    );

    expect(open).toEqual(["descriptor"]);
  });
});
