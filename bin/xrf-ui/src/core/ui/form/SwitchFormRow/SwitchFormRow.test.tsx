import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { SwitchFormRow } from "./SwitchFormRow";

describe("SwitchFormRow", () => {
  it("links the label and description to the controlled switch", async () => {
    const onChange = jest.fn();
    const { getByRole, getByTestId, getByText } = renderWithProviders(
      <SwitchFormRow
        data-testid={"sort-option"}
        id={"build-sort"}
        className={"build-option"}
        label={"Sort ids"}
        description={"Off preserves the order each source declares them in"}
        isChecked={false}
        onChange={onChange}
      />
    );
    const toggle = getByRole("switch", { name: "Sort ids" });

    expect(toggle).toHaveAttribute("id", "build-sort");
    expect(getByTestId("sort-option").closest(".build-option")).not.toBeNull();
    expect(toggle).toHaveAccessibleDescription("Off preserves the order each source declares them in");
    expect(toggle).not.toBeChecked();

    await userEvent.click(getByText("Sort ids"));

    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("marks a choice a run can do without, and holds still while disabled", () => {
    const { getByRole, getByText } = renderWithProviders(
      <SwitchFormRow label={"Replace existing text"} isChecked isDisabled isRequired={false} onChange={jest.fn()} />
    );

    expect(getByText("Optional")).toBeInTheDocument();
    expect(getByRole("switch", { name: /Replace existing text/ })).toBeDisabled();
  });
});
