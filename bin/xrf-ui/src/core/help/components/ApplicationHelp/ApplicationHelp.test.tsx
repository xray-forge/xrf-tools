import { describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ApplicationHelp } from "@/core/help/components/ApplicationHelp/ApplicationHelp";
import { HelpService } from "@/core/help/services/help";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("ApplicationHelp", () => {
  it("offers no affordance outside an application", () => {
    const { queryByLabelText } = renderWithProviders(<ApplicationHelp />, { route: "/" });

    expect(queryByLabelText("Help")).not.toBeInTheDocument();
  });

  it("offers no affordance where no help is authored yet", () => {
    const { queryByLabelText } = renderWithProviders(<ApplicationHelp />, { route: "/characters-explorer" });

    expect(queryByLabelText("Help")).not.toBeInTheDocument();
  });

  it("opens the current application's help from the caption button", async () => {
    const { getByLabelText, getByText, getByRole } = renderWithProviders(<ApplicationHelp />, {
      route: "/archives-explorer",
    });

    await userEvent.click(getByLabelText("Help"));

    expect(getByText("Archives explorer")).toBeInTheDocument();
    expect(getByRole("dialog", { name: "Archives explorer" })).toBeInTheDocument();
    expect(getByText("Typical workflow")).toBeInTheDocument();
  });

  it("opens when the help command runs", async () => {
    const container: Container = mockContainer();
    const { getByText } = renderWithProviders(<ApplicationHelp />, { container, route: "/archives-explorer" });

    // `F1` reaching this is the dispatcher's, and is tested there against the command rather than the dialog.
    act(() => container.get(HelpService).open());

    await waitFor(() => expect(getByText("Typical workflow")).toBeInTheDocument());
  });

  it("closes and returns to the tool", async () => {
    const { getByLabelText, getByText, queryByText } = renderWithProviders(<ApplicationHelp />, {
      route: "/archives-explorer",
    });

    await userEvent.click(getByLabelText("Help"));

    expect(getByText("Typical workflow")).toBeInTheDocument();

    await userEvent.click(getByLabelText("Close help"));

    await waitFor(() => expect(queryByText("Typical workflow")).not.toBeInTheDocument());

    expect(getByLabelText("Help")).toHaveFocus();
  });

  it("closes on Escape and restores focus to Help", async () => {
    const { getByLabelText, queryByRole } = renderWithProviders(<ApplicationHelp />, { route: "/archives-explorer" });
    const trigger = getByLabelText("Help");

    await userEvent.click(trigger);
    await userEvent.keyboard("{Escape}");

    await waitFor(() => expect(queryByRole("dialog")).not.toBeInTheDocument());

    expect(trigger).toHaveFocus();
  });
});
