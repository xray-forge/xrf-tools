import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { ChoiceFormRow, IChoiceFormRowOption } from "./ChoiceFormRow";

type TMode = "folder" | "model";

const OPTIONS: ReadonlyArray<IChoiceFormRowOption<TMode>> = [
  { value: "folder", label: "Folder", "aria-label": "Open folder" },
  { value: "model", label: "Model", "aria-label": "Open model" },
];

describe("ChoiceFormRow", () => {
  it("labels the group and requests a controlled selection through the keyboard", async () => {
    const onChange = jest.fn<(value: TMode) => void>();
    const { getByRole, getByTestId, rerender } = renderWithProviders(
      <ChoiceFormRow
        data-testid={"open-mode"}
        id={"visual-mode"}
        className={"visual-field"}
        label={"Open"}
        description={"Browse a folder or inspect one model"}
        options={OPTIONS}
        value={"folder"}
        onChange={onChange}
      />
    );
    const group = getByRole("group", { name: "Open" });

    expect(getByTestId("open-mode")).toBe(group);
    expect(group).toHaveAttribute("id", "visual-mode");
    expect(group).toHaveClass("visual-field");
    expect(group).toHaveAccessibleDescription("Browse a folder or inspect one model");
    expect(getByRole("button", { name: "Open folder", pressed: true })).toBeInTheDocument();

    await userEvent.tab();
    await userEvent.keyboard("{ArrowRight}");

    expect(getByRole("button", { name: "Open model" })).toHaveFocus();

    await userEvent.keyboard(" ");

    expect(onChange).toHaveBeenCalledWith("model");
    expect(getByRole("button", { name: "Open folder", pressed: true })).toBeInTheDocument();

    rerender(
      <>
        <ChoiceFormRow label={"Open"} options={OPTIONS} value={"model"} onChange={onChange} />
      </>
    );

    expect(getByRole("button", { name: "Open model", pressed: true })).toHaveFocus();
    expect(getByRole("button", { name: "Open folder", pressed: false })).toBeInTheDocument();
  });

  it("retains the active option when clicked again and accepts an empty-string value", async () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithProviders(
      <ChoiceFormRow
        label={"Layout"}
        options={[
          { value: "source", label: "Project sources" },
          { value: "", label: "Default layout" },
        ]}
        value={"source"}
        onChange={onChange}
      />
    );

    await userEvent.click(getByRole("button", { name: "Project sources" }));

    expect(onChange).not.toHaveBeenCalled();
    expect(getByRole("button", { name: "Project sources", pressed: true })).toBeInTheDocument();

    await userEvent.click(getByRole("button", { name: "Default layout" }));

    expect(onChange).toHaveBeenCalledWith("");
  });

  it("keeps each group's description separate and disables every choice while busy", async () => {
    const onChange = jest.fn();
    const { getByRole, getAllByRole } = renderWithProviders(
      <>
        <ChoiceFormRow
          label={"First layout"}
          description={"Read project sources"}
          options={[{ value: "source", label: "Sources" }]}
          value={"source"}
          onChange={onChange}
        />
        <ChoiceFormRow
          label={"Second layout"}
          description={"Read game data"}
          options={[{ value: "gamedata", label: "Game data" }]}
          value={"gamedata"}
          isDisabled
          onChange={onChange}
        />
      </>
    );
    const [first, second] = getAllByRole("group");

    expect(first.id).not.toBe(second.id);
    expect(first).toHaveAccessibleDescription("Read project sources");
    expect(second).toHaveAccessibleDescription("Read game data");
    expect(getByRole("button", { name: "Game data", pressed: true })).toBeDisabled();

    await userEvent.tab();

    expect(getByRole("button", { name: "Sources" })).toHaveFocus();

    await userEvent.tab();

    expect(getByRole("button", { name: "Game data" })).not.toHaveFocus();
    expect(onChange).not.toHaveBeenCalled();
  });
});
