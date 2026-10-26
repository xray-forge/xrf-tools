import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { CheckboxFormRow } from "./CheckboxFormRow";

describe("CheckboxFormRow", () => {
  it("links the label and description to the controlled input", async () => {
    const onChange = jest.fn();
    const { getByRole, getByText, getByTestId, rerender } = renderWithProviders(
      <CheckboxFormRow
        data-testid={"check-option"}
        id={"format-check"}
        className={"format-option"}
        label={"Check only"}
        description={"Report differences without rewriting files"}
        isChecked={true}
        onChange={onChange}
      />
    );
    const checkbox = getByRole("checkbox", { name: "Check only" });

    expect(checkbox).toHaveAttribute("id", "format-check");
    expect(getByTestId("check-option")).toHaveClass("format-option");
    expect(checkbox).toHaveAccessibleDescription("Report differences without rewriting files");
    expect(checkbox).toBeChecked();

    await userEvent.click(getByText("Check only"));

    expect(onChange).toHaveBeenCalledWith(false);
    expect(checkbox).toBeChecked();

    rerender(
      <>
        <CheckboxFormRow label={"Check only"} id={"format-check"} isChecked={false} onChange={onChange} />
      </>
    );

    expect(checkbox).not.toBeChecked();
    expect(checkbox).not.toHaveAttribute("aria-describedby");
  });

  it("requests a boolean change from the keyboard", async () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithProviders(<CheckboxFormRow label={"DLTX"} isChecked={false} onChange={onChange} />);

    await userEvent.tab();

    expect(getByRole("checkbox", { name: "DLTX" })).toHaveFocus();

    await userEvent.keyboard(" ");

    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("keeps generated associations separate and blocks the disabled input and label", async () => {
    const onChange = jest.fn();
    const { getAllByRole, getByRole, getByText } = renderWithProviders(
      <>
        <CheckboxFormRow label={"Check only"} description={"No writes"} isChecked={true} onChange={onChange} />
        <CheckboxFormRow
          label={"Strict"}
          description={"Fully decode sounds"}
          isChecked={false}
          isDisabled
          onChange={onChange}
        />
      </>
    );
    const [check, strict] = getAllByRole("checkbox");

    expect(check.id).not.toBe(strict.id);
    expect(check).toHaveAccessibleDescription("No writes");
    expect(strict).toHaveAccessibleDescription("Fully decode sounds");
    expect(strict).toBeDisabled();

    await userEvent.click(getByText("Strict"));
    await userEvent.tab();

    expect(getByRole("checkbox", { name: "Strict" })).not.toHaveFocus();
    expect(onChange).not.toHaveBeenCalled();
    expect(strict).not.toBeChecked();
  });
});
