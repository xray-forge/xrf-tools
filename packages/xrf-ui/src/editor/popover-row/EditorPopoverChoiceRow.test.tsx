import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { IChoiceFormRowOption } from "#/form/choice-row";
import { renderWithTheme } from "#/testing/render";

import { EditorPopoverChoiceRow } from "./EditorPopoverChoiceRow";

type TQuality = "low" | "medium" | "high";

const OPTIONS: ReadonlyArray<IChoiceFormRowOption<TQuality>> = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High", isDisabled: true },
];

describe("EditorPopoverChoiceRow", () => {
  it("names the group by its label, keeps one value pressed, and tells another chosen", async () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPopoverChoiceRow
        label={"Shadows"}
        description={"How sharp shadows are"}
        options={OPTIONS}
        value={"low"}
        onChange={onChange}
      />
    );
    const group: HTMLElement = getByRole("group", { name: "Shadows" });

    expect(group).toHaveAccessibleDescription("How sharp shadows are");
    expect(getByRole("button", { name: "Low" })).toHaveAttribute("aria-pressed", "true");
    expect(getByRole("button", { name: "High" })).toBeDisabled();

    await userEvent.click(getByRole("button", { name: "Medium" }));

    expect(onChange).toHaveBeenCalledWith("medium");
  });

  it("tells nothing when the pressed value is pressed again", async () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPopoverChoiceRow label={"Shadows"} options={OPTIONS} value={"low"} onChange={onChange} />
    );

    await userEvent.click(getByRole("button", { name: "Low" }));

    expect(onChange).not.toHaveBeenCalled();
  });
});
