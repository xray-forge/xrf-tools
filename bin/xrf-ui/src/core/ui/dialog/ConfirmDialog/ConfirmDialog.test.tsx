import { describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { ConfirmDialog } from "./ConfirmDialog";

describe("ConfirmDialog", () => {
  it.each(["Escape", "Cancel"])("starts on Cancel and restores focus after %s", async (action) => {
    const onConfirm = jest.fn();

    function Example(): ReactElement {
      const [isOpen, setOpen] = useState(false);

      return (
        <>
          <button onClick={() => setOpen(true)}>Discard edits</button>
          <ConfirmDialog
            isOpen={isOpen}
            title={"Discard edits?"}
            description={"Two files have unsaved changes."}
            confirmLabel={"Discard"}
            onConfirm={onConfirm}
            onClose={() => setOpen(false)}
          />
        </>
      );
    }

    const { getByRole, queryByRole } = renderWithProviders(<Example />);
    const trigger = getByRole("button", { name: "Discard edits" });

    await userEvent.click(trigger);

    expect(getByRole("dialog", { name: "Discard edits?" })).toHaveAccessibleDescription(
      "Two files have unsaved changes."
    );
    expect(getByRole("button", { name: "Cancel" })).toHaveFocus();

    if (action === "Escape") {
      await userEvent.keyboard("{Escape}");
    } else {
      await userEvent.keyboard("{Enter}");
    }

    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());

    expect(trigger).toHaveFocus();
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("holds confirmation until enabled and invokes it explicitly", async () => {
    const onConfirm = jest.fn();
    const onClose = jest.fn();
    const { getByRole, rerender } = renderWithProviders(
      <ConfirmDialog
        isOpen
        title={"Clear storage?"}
        description={"Removes remembered paths."}
        isConfirmDisabled
        onConfirm={onConfirm}
        onClose={onClose}
      />
    );

    expect(getByRole("button", { name: "Confirm" })).toBeDisabled();

    rerender(
      <>
        <ConfirmDialog
          isOpen
          title={"Clear storage?"}
          description={"Removes remembered paths."}
          onConfirm={onConfirm}
          onClose={onClose}
        />
      </>
    );

    await userEvent.click(getByRole("button", { name: "Confirm" }));

    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onClose).not.toHaveBeenCalled();
  });
});
