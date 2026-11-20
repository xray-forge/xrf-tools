import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { EditorSearchMenu } from "./EditorSearchMenu";

const ITEMS = [
  { id: "first", name: "file alpha" },
  { id: "second", name: "file beta" },
];

describe("EditorSearchMenu", () => {
  it("heads the menu as a panel, with its count and a field that filters it", () => {
    const view = renderWithProviders(
      <EditorSearchMenu
        title={"Files"}
        searchLabel={"Filter files"}
        resultsLabel={"File results"}
        items={ITEMS}
        toSearchText={(item) => item.name}
        toRow={(item) => ({ id: item.id, label: item.name })}
        onSelect={jest.fn()}
      />
    );

    expect(view.getByRole("heading", { name: "Files" })).toBeInTheDocument();
    expect(view.getByText("2")).toBeInTheDocument();

    fireEvent.change(view.getByRole("textbox", { name: "Filter files" }), { target: { value: "beta" } });

    expect(view.getByRole("button", { name: "file beta" })).toBeInTheDocument();
    expect(view.queryByRole("button", { name: "file alpha" })).not.toBeInTheDocument();
  });
});
