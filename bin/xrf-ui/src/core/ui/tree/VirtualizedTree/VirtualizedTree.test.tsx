import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { IPathTreeItem, parsePathTree, toDirectoryItemId, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree/VirtualizedTree";
import { renderWithProviders } from "@/fixtures/utils/render";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

const ICONS = { collapsed: <span>+</span>, expanded: <span>-</span>, leaf: <span>.</span> };

function getAt(...segments: Array<string>): string {
  return segments.join(LOGICAL_PATH_SEPARATOR);
}

function mockTree(): Array<IPathTreeItem<string>> {
  return parsePathTree(
    [
      { path: getAt("meshes", "ak74.ogf"), payload: "a" },
      { path: getAt("meshes", "pm.ogf"), payload: "b" },
      { path: "readme.txt", payload: "c" },
    ],
    LOGICAL_PATH_SEPARATOR
  );
}

interface IRenderOptions {
  expanded?: Array<string>;
  activeId?: string;
  onToggleExpanded?: (id: string) => void;
  onSelect?: (node: ITreeNode<string>) => void;
  onActivate?: (node: ITreeNode<string>) => void;
}

function Component({ expanded = [], activeId, onToggleExpanded, onSelect, onActivate }: IRenderOptions): ReactElement {
  const state: IUseTreeState = useTreeState({ initialExpandedIds: expanded });

  return (
    <VirtualizedTree<string>
      ariaLabel={"Visuals"}
      icons={ICONS}
      items={mockTree()}
      expandedIds={state.expandedIds}
      selectedId={state.selectedId}
      activeId={activeId}
      onSelect={(node: ITreeNode<string>) => {
        state.select(node.id);
        onSelect?.(node);
      }}
      onActivate={(node: ITreeNode<string>) => onActivate?.(node)}
      onToggleExpanded={(id: string) => {
        state.toggleExpanded(id);
        onToggleExpanded?.(id);
      }}
    />
  );
}

function render(options: IRenderOptions = {}): RenderResult {
  return renderWithProviders(<Component {...options} />);
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

  it("selects a directory on one click without opening it", async () => {
    const onToggleExpanded = jest.fn();
    const onActivate = jest.fn();
    const onSelect = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded, onActivate, onSelect });

    await userEvent.click(render_.getByText("meshes"));

    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ label: "meshes" }));
    expect(onToggleExpanded).not.toHaveBeenCalled();
    expect(onActivate).not.toHaveBeenCalled();
    expect(render_.getAllByRole("treeitem")[0]).toHaveAttribute("aria-selected", "true");
  });

  // The two states answer different questions, and they part company the moment somebody arrows past what is open.
  it("marks the open row apart from where the keyboard stands", async () => {
    const render_: RenderResult = render({
      activeId: toFileItemId("readme.txt"),
      expanded: [toDirectoryItemId("meshes")],
    });

    await userEvent.click(render_.getByText("ak74.ogf"));

    const rows: Array<HTMLElement> = render_.getAllByRole("treeitem");
    const open: HTMLElement = rows[3];
    const cursor: HTMLElement = rows[1];

    expect(open).toHaveAttribute("aria-current", "true");
    expect(open).toHaveAttribute("aria-selected", "false");

    expect(cursor).toHaveAttribute("aria-selected", "true");
    expect(cursor).not.toHaveAttribute("aria-current");
  });

  it("marks nothing as open for a tree whose rows open nothing", () => {
    const render_: RenderResult = render();

    expect(render_.getAllByRole("treeitem").every((it: HTMLElement) => !it.hasAttribute("aria-current"))).toBe(true);
  });

  it("selects a leaf on one click without opening it", async () => {
    const onActivate = jest.fn();
    const onSelect = jest.fn();
    const render_: RenderResult = render({ onActivate, onSelect });

    await userEvent.click(render_.getByText("readme.txt"));

    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({ label: "readme.txt" }));
    expect(onActivate).not.toHaveBeenCalled();
  });

  it("both opens and reports a directory on a double click", async () => {
    // Reporting it matters: the archive explorer selects a directory to extract it, and decides that
    // from the id rather than from a flag this component would have to carry.
    const onToggleExpanded = jest.fn();
    const onActivate = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded, onActivate });

    await userEvent.dblClick(render_.getByText("meshes"));

    expect(onToggleExpanded).toHaveBeenCalledWith(toDirectoryItemId("meshes"));
    expect(onActivate).toHaveBeenCalledWith(expect.objectContaining({ label: "meshes" }));
    expect(await render_.findByText("ak74.ogf")).toBeInTheDocument();
  });

  it("activates a leaf on a double click without toggling it", async () => {
    const onToggleExpanded = jest.fn();
    const onActivate = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded, onActivate });

    await userEvent.dblClick(render_.getByText("readme.txt"));

    expect(onActivate).toHaveBeenCalledWith(expect.objectContaining({ label: "readme.txt" }));
    expect(onToggleExpanded).not.toHaveBeenCalled();
  });

  it("opens a directory from the chevron without moving the selection", async () => {
    const onToggleExpanded = jest.fn();
    const onSelect = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded, onSelect });

    await userEvent.click(render_.getByText("readme.txt"));
    onSelect.mockClear();

    await userEvent.click(render_.getAllByTestId("virtualized-tree-chevron")[0]);

    expect(onToggleExpanded).toHaveBeenCalledWith(toDirectoryItemId("meshes"));
    expect(onSelect).not.toHaveBeenCalled();
    // The leaf clicked first is still the selected row, now below the two the directory revealed.
    expect(render_.getAllByRole("treeitem")[3]).toHaveAttribute("aria-selected", "true");
  });

  it("moves the selection with the arrow keys without moving focus off the tree", async () => {
    const onSelect = jest.fn();
    const onActivate = jest.fn();
    const render_: RenderResult = render({ onSelect, onActivate });
    const treeElement: HTMLElement = render_.getByRole("tree");

    treeElement.focus();
    await userEvent.keyboard("{ArrowDown}{ArrowDown}");

    // Focus stays put, which is the property that survives a row scrolling out of the rendered window.
    expect(treeElement).toHaveFocus();
    expect(onSelect).toHaveBeenLastCalledWith(expect.objectContaining({ label: "readme.txt" }));
    expect(onActivate).not.toHaveBeenCalled();

    const rows: Array<HTMLElement> = render_.getAllByRole("treeitem");

    expect(rows[1]).toHaveAttribute("aria-selected", "true");
    expect(treeElement).toHaveAttribute("aria-activedescendant", rows[1].id);
  });

  it("activates the selected row with enter, and does nothing with space", async () => {
    const onActivate = jest.fn();
    const onToggleExpanded = jest.fn();
    const render_: RenderResult = render({ onActivate, onToggleExpanded });

    render_.getByRole("tree").focus();
    await userEvent.keyboard("{ArrowDown}");
    await userEvent.keyboard(" ");

    // Space is unbound on purpose: it is what toggles membership once a tree selects more than one row.
    expect(onActivate).not.toHaveBeenCalled();
    expect(onToggleExpanded).not.toHaveBeenCalled();

    await userEvent.keyboard("{Enter}");

    expect(onActivate).toHaveBeenCalledWith(expect.objectContaining({ label: "meshes" }));
    expect(onToggleExpanded).toHaveBeenCalledWith(toDirectoryItemId("meshes"));
  });

  it("opens a closed directory with the right arrow", async () => {
    const onToggleExpanded = jest.fn();
    const render_: RenderResult = render({ onToggleExpanded });

    render_.getByRole("tree").focus();
    await userEvent.keyboard("{ArrowDown}{ArrowRight}");

    expect(onToggleExpanded).toHaveBeenCalledWith(toDirectoryItemId("meshes"));
  });

  it("steps to the next row with the right arrow once there is nothing left to open", async () => {
    const onToggleExpanded = jest.fn();
    const onSelect = jest.fn();
    const render_: RenderResult = render({ expanded: [toDirectoryItemId("meshes")], onToggleExpanded, onSelect });

    render_.getByRole("tree").focus();
    // From the open directory into its first child, then past that leaf to its sibling.
    await userEvent.keyboard("{ArrowDown}{ArrowRight}{ArrowRight}");

    expect(onSelect).toHaveBeenLastCalledWith(expect.objectContaining({ label: "pm.ogf" }));
    expect(onToggleExpanded).not.toHaveBeenCalled();
  });

  it("holds the selection on the last row when the right arrow has nowhere to step", async () => {
    const onSelect = jest.fn();
    const render_: RenderResult = render({ onSelect });

    render_.getByRole("tree").focus();
    await userEvent.keyboard("{ArrowDown}{ArrowDown}{ArrowRight}");

    expect(onSelect).toHaveBeenLastCalledWith(expect.objectContaining({ label: "readme.txt" }));
  });

  // Windowing itself is not asserted here on purpose: jsdom lays nothing out, and the virtualizer's
  // measurement runs through a throttled ResizeObserver, so every row renders no matter what the
  // element is told its size is. Row count, the focus ring, and the chevron column are verified
  // against the running app instead.
});
