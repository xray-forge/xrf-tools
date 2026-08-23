import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { IPathTreeItem, LOGICAL_PATH_SEPARATOR, parsePathTree, toDirectoryItemId } from "@/core/ui/tree/path-tree";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree/VirtualizedTree";
import { renderWithProviders } from "@/fixtures/utils/render";

const ICONS = { collapsed: <span>+</span>, expanded: <span>-</span>, leaf: <span>.</span> };

function at(...segments: Array<string>): string {
  return segments.join(LOGICAL_PATH_SEPARATOR);
}

function tree(): Array<IPathTreeItem<string>> {
  return parsePathTree(
    [
      { path: at("meshes", "ak74.ogf"), payload: "a" },
      { path: at("meshes", "pm.ogf"), payload: "b" },
      { path: "readme.txt", payload: "c" },
    ],
    LOGICAL_PATH_SEPARATOR
  );
}

interface IRenderOptions {
  expanded?: Array<string>;
  onToggleExpanded?: () => void;
  onSelect?: () => void;
}

function render({
  expanded = [],
  onToggleExpanded = jest.fn(),
  onSelect = jest.fn(),
}: IRenderOptions = {}): RenderResult {
  return renderWithProviders(
    <VirtualizedTree<string>
      ariaLabel={"Visuals"}
      icons={ICONS}
      items={tree()}
      expandedIds={new Set(expanded)}
      selectedId={null}
      onSelect={onSelect}
      onToggleExpanded={onToggleExpanded}
    />
  );
}

describe("VirtualizedTree", () => {
  it("exposes itself as a tree of rows", () => {
    const render_: RenderResult = render();

    expect(render_.getByRole("tree", { name: "Visuals" })).toBeInTheDocument();
    expect(render_.getAllByRole("treeitem")).toHaveLength(2);
  });

  it("states the depth and sibling position that a flat DOM no longer implies", () => {
    const render_: RenderResult = render({ expanded: [toDirectoryItemId("meshes")] });
    const rows: Array<HTMLElement> = render_.getAllByRole("treeitem");

    expect(rows.map((it: HTMLElement) => it.getAttribute("aria-level"))).toEqual(["1", "2", "2", "1"]);
    expect(rows.map((it: HTMLElement) => it.getAttribute("aria-posinset"))).toEqual(["1", "1", "2", "2"]);
    expect(rows[0]).toHaveAttribute("aria-expanded", "true");
    // A leaf must not claim it can be opened.
    expect(rows[1]).not.toHaveAttribute("aria-expanded");
  });

  it("both opens and reports a directory", async () => {
    // Reporting it matters: the archive explorer selects a directory to extract it, and decides that
    // from the id rather than from a flag this component would have to carry.
    const onToggleExpanded = jest.fn();
    const onSelect = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded, onSelect });

    await userEvent.click(render_.getByText("meshes"));

    expect(onToggleExpanded).toHaveBeenCalledWith(toDirectoryItemId("meshes"));
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ label: "meshes", kind: "directory" }));
  });

  it("selects a leaf rather than toggling it", async () => {
    const onToggleExpanded = jest.fn();
    const onSelect = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded, onSelect });

    await userEvent.click(render_.getByText("readme.txt"));

    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ label: "readme.txt", kind: "file" }));
    expect(onToggleExpanded).not.toHaveBeenCalled();
  });

  it("moves the keyboard position without moving focus off the tree", async () => {
    const render_: RenderResult = render();
    const treeElement: HTMLElement = render_.getByRole("tree");
    const first: string = treeElement.getAttribute("aria-activedescendant") ?? "";

    treeElement.focus();
    await userEvent.keyboard("{ArrowDown}");

    // Focus stays put, which is the property that survives a row scrolling out of the window.
    expect(treeElement).toHaveFocus();
    expect(treeElement.getAttribute("aria-activedescendant")).not.toBe(first);
  });

  it("opens a closed directory with the right arrow", async () => {
    const onToggleExpanded = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded });

    render_.getByRole("tree").focus();
    await userEvent.keyboard("{ArrowRight}");

    expect(onToggleExpanded).toHaveBeenCalledWith(toDirectoryItemId("meshes"));
  });

  // Windowing itself is not asserted here on purpose: jsdom lays nothing out, and the virtualizer's
  // measurement runs through a throttled ResizeObserver, so every row renders no matter what the
  // element is told its size is. Row count is verified against the running app instead.
});
