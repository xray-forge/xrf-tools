import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { LtxInventoryFile } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsMenu } from "@/core/ltx/components/ConfigsMenu/ConfigsMenu";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

/** The committed fixture's shape: an entry point and the config it includes, one directory deep. */
const FILES: Array<LtxInventoryFile> = [
  { isPhysical: true, path: "configs\\system.ltx", role: { kind: "entryPoint" }, source: "gamedata" },
  {
    isPhysical: true,
    path: "configs\\items\\w_base.ltx",
    role: { kind: "included", by: ["configs\\system.ltx"] },
    source: "gamedata",
  },
];

function renderMenu(onOpen: (path: string) => void, selected: Nullable<string> = null): RenderResult {
  return renderWithProviders(<ConfigsMenu files={FILES} selected={selected} onOpen={onOpen} />);
}

function getRowOf(render: RenderResult, label: string): HTMLElement {
  return render.getByText(label).closest("[role='treeitem']") as HTMLElement;
}

describe("ConfigsMenu", () => {
  it("selects a config on one click without reading it", () => {
    // Selecting is inert by contract, and here that is what keeps arrow keys usable: `onSelect` fires on every move,
    // so an opening selection would resolve a config per keystroke down a directory.
    const onOpen = jest.fn();
    const render = renderMenu(onOpen);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.click(render.getByText("system.ltx"));

    expect(onOpen).not.toHaveBeenCalled();
    expect(getRowOf(render, "system.ltx")).toHaveAttribute("aria-selected", "true");
  });

  it("opens a config on a double click, as every other explorer tree does", () => {
    const onOpen = jest.fn();
    const render = renderMenu(onOpen);

    fireEvent.dblClick(render.getByText("configs"));
    fireEvent.dblClick(render.getByText("system.ltx"));

    expect(onOpen).toHaveBeenCalledWith("configs\\system.ltx");
  });

  it("leaves a directory to its chevron, since it stands for no config", () => {
    const onOpen = jest.fn();
    const render = renderMenu(onOpen);

    fireEvent.dblClick(render.getByText("configs"));

    expect(onOpen).not.toHaveBeenCalled();
    expect(render.getByText("items")).toBeInTheDocument();
  });

  it("reveals whatever is on screen, whoever opened it", () => {
    // The tree is not the only opener: a finding in the Problems panel jumps straight to a config, and the row it
    // lands on has to be expanded to and selected rather than left behind a closed directory.
    const render = renderMenu(jest.fn(), "configs\\items\\w_base.ltx");

    expect(getRowOf(render, "w_base.ltx")).toHaveAttribute("aria-selected", "true");
  });
});
