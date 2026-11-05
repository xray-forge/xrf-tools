import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { MenuItem } from "#/material/menu";
import { renderWithTheme } from "#/testing/render";

import { EditorPopoverSelectRow } from "./EditorPopoverSelectRow";

describe("EditorPopoverSelectRow", () => {
  it("names the select by its label alone, describes it, and focuses it from the label", async () => {
    const onChange = jest.fn();
    const { getByRole, getByText } = renderWithTheme(
      <EditorPopoverSelectRow
        label={"Cycle"}
        description={"The weather cycle played"}
        value={"default_clear"}
        onChange={onChange}
      >
        <MenuItem value={"default_clear"}>default_clear</MenuItem>
        <MenuItem value={"default_rain"}>default_rain</MenuItem>
      </EditorPopoverSelectRow>
    );
    const select: HTMLElement = getByRole("combobox", { name: "Cycle" });

    expect(select).toHaveAccessibleDescription("The weather cycle played");
    expect(select).toHaveTextContent("default_clear");

    await userEvent.click(getByText("Cycle"));

    expect(select).toHaveFocus();

    await userEvent.click(select);
    await userEvent.click(getByRole("option", { name: "default_rain" }));

    expect(onChange).toHaveBeenCalledWith("default_rain");
  });

  it("reads an empty value as the caller says, and blocks a disabled one", () => {
    const { getByRole } = renderWithTheme(
      <EditorPopoverSelectRow
        label={"Flare set"}
        value={""}
        renderValue={(it) => it || "None"}
        isDisabled
        onChange={() => {}}
      >
        <MenuItem value={""}>None</MenuItem>
      </EditorPopoverSelectRow>
    );
    const select: HTMLElement = getByRole("combobox", { name: "Flare set" });

    expect(select).toHaveTextContent("None");
    expect(select).toHaveAttribute("aria-disabled", "true");
  });
});
