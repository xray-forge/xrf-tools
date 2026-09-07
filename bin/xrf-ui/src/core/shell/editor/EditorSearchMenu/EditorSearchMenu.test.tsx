import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { ReactElement } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorSearchMenu } from "./EditorSearchMenu";

const ITEMS = [
  { id: "first", name: "file alpha", detail: "first source" },
  { id: "second", name: "file beta", detail: "second source" },
  { id: "third", name: "file gamma", detail: "third source" },
];

describe("EditorSearchMenu", () => {
  it("opens the original item through both keyboard and pointer activation", () => {
    const onSelect = jest.fn();
    const view = renderWithProviders(
      <EditorSearchMenu
        title={"Files"}
        searchLabel={"Filter files"}
        resultsLabel={"File results"}
        items={ITEMS}
        toSearchText={(item) => item.name}
        toRow={(item) => ({ id: item.id, label: item.name, description: item.detail })}
        onSelect={onSelect}
      >
        <div>Default tree</div>
      </EditorSearchMenu>
    );
    const input = view.getByRole("textbox", { name: "Filter files" });

    fireEvent.change(input, { target: { value: "file" } });
    fireEvent.keyDown(input, { key: "ArrowDown" });

    expect(view.getByRole("button", { name: "file beta second source" })).toHaveClass("Mui-selected");
    expect(view.queryByText("Default tree")).not.toBeInTheDocument();

    fireEvent.keyDown(input, { key: "Enter" });

    expect(onSelect.mock.calls[0][0]).toBe(ITEMS[1]);

    fireEvent.click(view.getByText("file alpha"));

    expect(onSelect.mock.calls[1][0]).toBe(ITEMS[0]);
  });

  it("reports capped and empty results and restores the default content when cleared", () => {
    const view = renderWithProviders(
      <EditorSearchMenu
        title={"Files"}
        searchLabel={"Filter files"}
        resultsLabel={"File results"}
        items={ITEMS}
        limit={2}
        toSearchText={(item) => item.name}
        toRow={(item) => ({ id: item.id, label: item.name })}
        onSelect={jest.fn()}
        header={<button>Extra filter</button>}
      >
        <div>Default tree</div>
      </EditorSearchMenu>
    );
    const input = view.getByRole("textbox", { name: "Filter files" });

    fireEvent.change(input, { target: { value: "file" } });

    expect(view.getByText("Showing 2 of 3 matches")).toBeInTheDocument();
    expect(view.getByRole("button", { name: "Extra filter" })).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "missing" } });

    expect(view.getByText("No files match missing.")).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "" } });

    expect(view.getByText("Default tree")).toBeInTheDocument();
    expect(view.queryByRole("list", { name: "File results" })).not.toBeInTheDocument();
  });

  it("keeps the highlight and Enter aligned when a filter reduces the dataset", () => {
    const onSelect = jest.fn();

    function renderMenu(items: typeof ITEMS): ReactElement {
      return (
        <EditorSearchMenu
          title={"Files"}
          searchLabel={"Filter files"}
          resultsLabel={"File results"}
          items={items}
          toSearchText={(item) => item.name}
          toRow={(item) => ({ id: item.id, label: item.name })}
          onSelect={onSelect}
        />
      );
    }
    const view = renderWithProviders(renderMenu(ITEMS));
    const input = view.getByRole("textbox", { name: "Filter files" });

    fireEvent.change(input, { target: { value: "file" } });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "ArrowDown" });
    view.rerender(<>{renderMenu([ITEMS[0], ITEMS[1]])}</>);

    expect(view.getByRole("button", { name: "file beta" })).toHaveClass("Mui-selected");

    fireEvent.keyDown(input, { key: "ArrowUp" });

    expect(view.getByRole("button", { name: "file alpha" })).toHaveClass("Mui-selected");

    fireEvent.keyDown(input, { key: "Enter" });

    expect(onSelect.mock.calls[0][0]).toBe(ITEMS[0]);
  });
});
