import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { IPathField } from "@/core/ui/form/use-path-field";
import { renderWithProviders } from "@/fixtures/utils/render";

import { PathPanelRow } from "./PathPanelRow";

/** Supplies the field contract without starting filesystem validation. */
function mockField(overrides: Partial<IPathField> = {}): IPathField {
  return {
    value: null,
    error: null,
    isValid: true,
    select: jest.fn<() => Promise<void>>().mockResolvedValue(undefined),
    clear: jest.fn(),
    setValue: jest.fn(),
    commit: jest.fn(),
    recents: { records: [], pick: jest.fn(), forget: jest.fn() },
    ...overrides,
  };
}

describe("PathPanelRow", () => {
  it("names the field by its label and describes it, adding what the validation found", () => {
    const field: IPathField = mockField({ value: "C:\\height.png" });
    const { getByRole, getByText, rerender } = renderWithProviders(
      <PathPanelRow label={"Height map"} description={"Averaged across its channels"} field={field} />
    );

    expect(getByRole("textbox", { name: "Height map" })).toHaveAccessibleDescription("Averaged across its channels");
    expect(getByRole("textbox", { name: "Height map" })).toHaveAttribute("aria-invalid", "false");

    rerender(
      <PathPanelRow
        label={"Height map"}
        description={"Averaged across its channels"}
        field={{ ...field, error: "Path does not exist", isValid: false }}
      />
    );

    expect(getByText("Path does not exist")).toBeInTheDocument();
    expect(getByRole("textbox", { name: "Height map" })).toHaveAccessibleDescription(
      "Averaged across its channels Path does not exist"
    );
    expect(getByRole("textbox", { name: "Height map" })).toHaveAttribute("aria-invalid", "true");
  });

  it("forwards editing, browsing, and clearing to the supplied field, and commits nothing itself", async () => {
    const field: IPathField = mockField({ value: "C:\\height.png" });
    const { getByRole } = renderWithProviders(<PathPanelRow label={"Height map"} field={field} />);

    fireEvent.change(getByRole("textbox", { name: "Height map" }), { target: { value: "D:\\relief.png" } });
    await userEvent.click(getByRole("button", { name: "Browse" }));
    await userEvent.click(getByRole("button", { name: "Clear" }));

    expect(field.setValue).toHaveBeenCalledWith("D:\\relief.png");
    expect(field.select).toHaveBeenCalledTimes(1);
    expect(field.clear).toHaveBeenCalledTimes(1);
    expect(field.commit).not.toHaveBeenCalled();
  });
});
