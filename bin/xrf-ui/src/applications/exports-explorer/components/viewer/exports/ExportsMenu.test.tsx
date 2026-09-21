import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ExportDescriptor } from "@/core/ipc/types/xrf-export";
import { mockExportsDeclarations } from "@/fixtures/mocks/project.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { groupExports } from "./exports-groups";
import { ExportsMenu } from "./ExportsMenu";

function renderMenu(onSelect: (name: string) => void): RenderResult {
  const declarations: Array<ExportDescriptor> = mockExportsDeclarations();

  return renderWithProviders(
    <ExportsMenu
      declarations={declarations}
      groups={groupExports(declarations)}
      selectedName={null}
      onSelect={onSelect}
    />
  );
}

describe("ExportsMenu", () => {
  it("shows a declaration on a double click, never on a single one", async () => {
    const onSelect = jest.fn();
    const render: RenderResult = renderMenu(onSelect);

    fireEvent.dblClick(render.getByText("xr_effects (1)"));

    const leaf: HTMLElement = await render.findByText("xr_effects.play_sound");

    fireEvent.click(leaf);

    expect(onSelect).not.toHaveBeenCalled();
    expect(leaf.closest("[role='treeitem']")).toHaveAttribute("aria-selected", "true");

    fireEvent.dblClick(leaf);

    expect(onSelect).toHaveBeenCalledWith("xr_effects.play_sound");
  });
});
