import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { EditorToolbarLocation } from "@/core/shell/editor/EditorToolbarLocation";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("EditorToolbarLocation", () => {
  it("shows the one path that opens a loose file", () => {
    const render: RenderResult = renderWithProviders(
      <EditorToolbarLocation location={{ path: "C:\\gamedata\\textures\\ston\\ston_beton05.dds" }} />
    );

    expect(render.getByTestId("editor-toolbar-location")).toHaveTextContent(
      "C:\\gamedata\\textures\\ston\\ston_beton05.dds"
    );
  });

  it("names the archive a packed session was opened on, and nothing inside it", () => {
    // What is open inside the volume is the editor's own header. The crumb stays on the volume, so it does not move as
    // a person clicks through the tree.
    const render: RenderResult = renderWithProviders(
      <EditorToolbarLocation location={{ path: "D:\\game\\db\\gamedata.db0" }} />
    );

    expect(render.getByTestId("editor-toolbar-location")).toHaveTextContent("D:\\game\\db\\gamedata.db0");
  });
});
