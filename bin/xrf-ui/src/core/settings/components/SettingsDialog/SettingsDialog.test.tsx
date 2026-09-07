import { describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement, useState } from "react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { SettingsDialog } from "./SettingsDialog";

function SettingsExample(): ReactElement {
  const [isOpen, setOpen] = useState(false);

  return (
    <>
      <button onClick={() => setOpen(true)}>Open settings</button>
      <SettingsDialog isOpen={isOpen} onClose={() => setOpen(false)} />
    </>
  );
}

describe("SettingsDialog", () => {
  it.each(["Escape", "Close settings", "Done"])("has a named dialog and restores focus after %s", async (action) => {
    const { getByRole, queryByRole } = renderWithProviders(<SettingsExample />);
    const trigger = getByRole("button", { name: "Open settings" });

    await userEvent.click(trigger);

    expect(getByRole("dialog", { name: "Settings" })).toBeInTheDocument();
    expect(getByRole("heading", { name: "Settings" })).toBeInTheDocument();
    expect(getByRole("button", { name: "Close settings" })).toBeInTheDocument();

    if (action === "Escape") {
      await userEvent.keyboard("{Escape}");
    } else {
      await userEvent.click(getByRole("button", { name: action }));
    }

    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());

    expect(trigger).toHaveFocus();
  });
});
