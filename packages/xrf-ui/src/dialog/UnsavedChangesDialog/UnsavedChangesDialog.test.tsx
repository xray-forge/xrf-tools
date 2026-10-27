import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { UnsavedChangesDialog } from "./UnsavedChangesDialog";

import { renderWithTheme } from "#/testing/render";

describe("UnsavedChangesDialog", () => {
  it("explains what is pending and focuses Stay", async () => {
    const onClose = jest.fn();
    const onSave = jest.fn();
    const onDiscard = jest.fn();
    const { getByRole } = renderWithTheme(
      <UnsavedChangesDialog
        isOpen
        description={"The current file has edits."}
        onSave={onSave}
        onDiscard={onDiscard}
        onClose={onClose}
      />
    );

    expect(getByRole("dialog", { name: "Leave without saving?" })).toHaveAccessibleDescription(
      "The current file has edits."
    );
    expect(getByRole("button", { name: "Stay" })).toHaveFocus();

    await userEvent.keyboard("{Escape}");

    expect(onClose).toHaveBeenCalledTimes(1);
    expect(onSave).not.toHaveBeenCalled();
    expect(onDiscard).not.toHaveBeenCalled();
  });

  it("keeps dismissal and actions unavailable during a save", async () => {
    const onClose = jest.fn();
    const { getByRole } = renderWithTheme(
      <UnsavedChangesDialog
        isOpen
        isSaving
        description={"Saving pending edits."}
        onSave={jest.fn()}
        onDiscard={jest.fn()}
        onClose={onClose}
      />
    );

    expect(getByRole("button", { name: "Stay" })).toBeDisabled();
    expect(getByRole("button", { name: "Discard and leave" })).toBeDisabled();
    expect(getByRole("button", { name: "Saving" })).toBeDisabled();

    await userEvent.keyboard("{Escape}");

    expect(onClose).not.toHaveBeenCalled();
  });
});
