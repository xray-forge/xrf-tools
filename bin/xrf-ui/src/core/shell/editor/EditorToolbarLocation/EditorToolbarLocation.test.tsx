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

  it("shows both addresses of a packed file", () => {
    // What a person needs from a file inside a volume is which volume, and what it is called in there.
    const render: RenderResult = renderWithProviders(
      <EditorToolbarLocation
        location={{ entry: "config\\gameplay\\character_criticals_1.xml", path: "D:\\game\\db\\gamedata.db0" }}
      />
    );

    const rendered: HTMLElement = render.getByTestId("editor-toolbar-location");

    expect(rendered).toHaveTextContent("D:\\game\\db\\gamedata.db0");
    expect(rendered).toHaveTextContent("config\\gameplay\\character_criticals_1.xml");
  });
});
