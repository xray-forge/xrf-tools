import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverCheckboxRow } from "./EditorPopoverCheckboxRow";

describe("EditorPopoverCheckboxRow", () => {
  it("names the box by its label, describes it by what it does, and turns it over from the label", async () => {
    const onChange = jest.fn();
    const { getByRole, getByText } = renderWithTheme(
      <EditorPopoverCheckboxRow
        label={"Refined"}
        description={"Settles each march where it met the ground"}
        isChecked={false}
        onChange={onChange}
      />
    );
    const checkbox = getByRole("checkbox", { name: "Refined" });

    expect(checkbox).toHaveAccessibleDescription("Settles each march where it met the ground");

    await userEvent.click(getByText("Refined"));

    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("keeps two rows' names apart and blocks a disabled one", async () => {
    const onChange = jest.fn();
    const { getByRole, getByText } = renderWithTheme(
      <>
        <EditorPopoverCheckboxRow label={"Refined"} isChecked={true} onChange={onChange} />
        <EditorPopoverCheckboxRow label={"Steep parallax"} isChecked={false} isDisabled onChange={onChange} />
      </>
    );

    expect(getByRole("checkbox", { name: "Refined" })).toBeChecked();
    expect(getByRole("checkbox", { name: "Steep parallax" })).toBeDisabled();
    expect(getByRole("checkbox", { name: "Steep parallax" })).not.toHaveAttribute("aria-describedby");

    await userEvent.click(getByText("Steep parallax"));

    expect(onChange).not.toHaveBeenCalled();
  });
});
