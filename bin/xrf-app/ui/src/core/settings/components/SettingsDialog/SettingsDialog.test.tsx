import { describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SettingsDialog } from "./SettingsDialog";

function SettingsExample(): ReactElement {
  const settingsDialogService: SettingsDialogService = useInjection(SettingsDialogService);

  return (
    <>
      <button onClick={settingsDialogService.open}>Open settings</button>
      <SettingsDialog />
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
