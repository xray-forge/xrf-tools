import { describe, expect, it } from "@jest/globals";
import { fireEvent, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ApplicationHelp } from "@/core/help/components/ApplicationHelp/ApplicationHelp";
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

  it("opens on F1 like every desktop application", async () => {
    const { getByText } = renderWithProviders(<ApplicationHelp />, { route: "/archives-explorer" });

    fireEvent.keyDown(window, { key: "F1" });

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
