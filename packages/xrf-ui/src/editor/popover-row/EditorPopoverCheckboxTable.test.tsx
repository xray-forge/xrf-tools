import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverCheckboxTable } from "./EditorPopoverCheckboxTable";
import { EditorPopoverSwitchRow } from "./EditorPopoverSwitchRow";

describe("EditorPopoverCheckboxTable", () => {
  it("names each box by its row and column, leaves a row's missing cell empty and blocks a disabled one", async () => {
    const onShown = jest.fn();
    const onHelper = jest.fn();
    const { getByRole, queryByRole } = renderWithTheme(
      <EditorPopoverCheckboxTable
        columns={[
          { icon: <span />, id: "shown", label: "Shown" },
          { icon: <span />, id: "helper", label: "Helper" },
        ]}
        rows={[
          {
            cells: [
              { isChecked: true, onChange: onShown },
              { isChecked: false, isDisabled: true, onChange: onHelper, reason: "Helpers are off" },
            ],
            id: "items",
            label: "Items",
          },
          { cells: [null, { isChecked: true, onChange: onHelper }], id: "restrictors", label: "Restrictors" },
        ]}
      />
    );

    await userEvent.click(getByRole("checkbox", { name: "Items shown" }));

    expect(onShown).toHaveBeenCalledWith(false);
    expect(getByRole("checkbox", { name: "Items helper" })).toBeDisabled();
    expect(queryByRole("checkbox", { name: "Restrictors shown" })).toBeNull();
    expect(getByRole("checkbox", { name: "Restrictors helper" })).toBeChecked();
  });
});

describe("EditorPopoverSwitchRow", () => {
  it("names its box by its label, describes it, and turns it over from the label", async () => {
    const onChange = jest.fn();
    const { getByRole, getByText } = renderWithTheme(
      <EditorPopoverSwitchRow
        label={"Grid"}
        description={"Draws the ground plane"}
        isChecked={false}
        onChange={onChange}
      />
    );

    expect(getByRole("checkbox", { name: "Grid" })).toHaveAccessibleDescription("Draws the ground plane");

    await userEvent.click(getByText("Grid"));

    expect(onChange).toHaveBeenCalledWith(true);
  });
});
