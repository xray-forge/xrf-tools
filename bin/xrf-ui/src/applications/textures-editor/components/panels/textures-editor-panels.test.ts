import { describe, expect, it } from "@jest/globals";

import { IEditorPanel } from "@/core/shell/panel/context";

import { createTexturesEditorPanels } from "./textures-editor-panels";

describe("createTexturesEditorPanels", () => {
  it("registers no tree, whatever the session holds", () => {
    // The whole difference between this tool and the explorer. A workbench works one texture at a time; a listing
    // beside it would make it a browser that happens to write, which is what this application exists not to be.
    const panels: Array<IEditorPanel> = createTexturesEditorPanels();

    expect(panels.map((panel: IEditorPanel) => panel.id)).toEqual([
      "descriptor",
      "formats",
      "bump",
      "material",
      "files",
      "channels",
    ]);
    expect(panels.some((panel: IEditorPanel) => panel.side === "left")).toBe(false);
  });

  it("opens the descriptor first, because that is what this tool is for", () => {
    // The explorer opens `Material` instead: the same `.thm` read from the other end. Which one is in front is the
    // difference between a tool for understanding a texture and a tool for changing one.
    const open: Array<string> = createTexturesEditorPanels()
      .filter((panel: IEditorPanel) => panel.isOpenByDefault)
      .map((panel: IEditorPanel) => panel.id);

    expect(open).toEqual(["descriptor"]);
  });
});
