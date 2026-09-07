import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { AssetRootFormRow } from "@/core/assets/components/AssetRootFormRow";
import { PickerForm } from "@/core/shell/editor/PickerForm";
import { IPathField } from "@/core/ui/form/use-path-field";
import { renderWithProviders } from "@/fixtures/utils/render";

import { PathFormRow } from "./PathFormRow";

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

describe("PathFormRow", () => {
  it("associates the visible description and message with the input as validation changes", () => {
    const field: IPathField = mockField({ value: "C:\\game" });
    const { getByRole, queryByText, rerender } = renderWithProviders(
      <PathFormRow label={"Source"} description={"Directory to read"} fact={"Game installation"} field={field} />
    );
    const input: HTMLElement = getByRole("textbox", { name: "Source" });
    const id: string = input.id;

    expect(input).toHaveAccessibleDescription("Directory to read Game installation");
    expect(input).toHaveAttribute("aria-invalid", "false");

    rerender(
      <>
        <PathFormRow
          label={"Source"}
          description={"Directory to read"}
          fact={"Game installation"}
          field={{ ...field, error: "Path does not exist", isValid: false }}
        />
      </>
    );

    expect(getByRole("textbox", { name: "Source" })).toHaveAttribute("id", id);
    expect(getByRole("textbox", { name: "Source" })).toHaveAccessibleDescription(
      "Directory to read Path does not exist"
    );
    expect(getByRole("textbox", { name: "Source" })).toHaveAttribute("aria-invalid", "true");
    expect(queryByText("Game installation")).not.toBeInTheDocument();

    rerender(
      <>
        <PathFormRow label={"Source"} field={field} />
      </>
    );

    expect(getByRole("textbox", { name: "Source" })).not.toHaveAttribute("aria-describedby");
    expect(getByRole("textbox", { name: "Source" })).toHaveAttribute("aria-invalid", "false");
  });

  it("keeps labels and descriptions local to each field", async () => {
    const { getByRole, getByText } = renderWithProviders(
      <>
        <PathFormRow label={"Source"} description={"Directory to read"} field={mockField()} />
        <PathFormRow label={"Output"} description={"Directory to write"} field={mockField()} />
      </>
    );
    const source: HTMLElement = getByRole("textbox", { name: "Source" });
    const output: HTMLElement = getByRole("textbox", { name: "Output" });

    expect(source.id).not.toBe(output.id);
    expect(source).toHaveAccessibleDescription("Directory to read");
    expect(output).toHaveAccessibleDescription("Directory to write");

    await userEvent.click(getByText("Output"));

    expect(output).toHaveFocus();
  });

  it("forwards editing, browsing, and clearing to the supplied field", async () => {
    const field: IPathField = mockField({ value: "C:\\game" });
    const { getByRole } = renderWithProviders(<PathFormRow label={"Source"} field={field} />);

    fireEvent.change(getByRole("textbox", { name: "Source" }), { target: { value: "D:\\mod" } });
    await userEvent.click(getByRole("button", { name: "Browse" }));
    await userEvent.click(getByRole("button", { name: "Clear" }));

    expect(field.setValue).toHaveBeenCalledWith("D:\\mod");
    expect(field.select).toHaveBeenCalledTimes(1);
    expect(field.clear).toHaveBeenCalledTimes(1);
    expect(field.commit).not.toHaveBeenCalled();
  });

  it("commits ordinary and asset-root fields once before submitting and unregisters removed rows", async () => {
    const calls: Array<string> = [];
    const field: IPathField = mockField({ commit: () => calls.push("source") });
    const root: IPathField = mockField({ commit: () => calls.push("root") });

    function onSubmit(): void {
      calls.push("submit");
    }
    const { getByRole, rerender } = renderWithProviders(
      <PickerForm title={"Open"} submitLabel={"Open project"} onSubmit={onSubmit}>
        <PathFormRow label={"Source"} field={field} />
        <AssetRootFormRow field={root} />
      </PickerForm>
    );

    await userEvent.click(getByRole("button", { name: "Open project" }));

    expect(calls).toEqual(["source", "root", "submit"]);

    rerender(
      <>
        <PickerForm title={"Open"} submitLabel={"Open project"} onSubmit={onSubmit} />
      </>
    );

    await userEvent.click(getByRole("button", { name: "Open project" }));

    expect(calls).toEqual(["source", "root", "submit", "submit"]);
  });
});
