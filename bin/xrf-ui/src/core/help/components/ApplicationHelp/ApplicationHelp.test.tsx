import { describe, expect, it } from "@jest/globals";
import { act, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { HelpService } from "@/core/help/services/help";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ApplicationHelp } from "./ApplicationHelp";

describe("ApplicationHelp", () => {
  it("offers the home screen's own help outside an application", async () => {
    const { getByLabelText, getByRole } = renderWithProviders(<ApplicationHelp />, { route: "/" });

    await userEvent.click(getByLabelText("Help"));

    expect(getByRole("dialog", { name: "XRF Tools" })).toBeInTheDocument();
  });

  it("offers a disabled affordance on a route that is neither", () => {
    const { getByLabelText } = renderWithProviders(<ApplicationHelp />, { route: "/not-a-tool" });

    expect(getByLabelText("Help")).toBeDisabled();
  });

  it("offers a disabled affordance where no help is authored yet, saying so", () => {
    const { getByLabelText, getByTitle } = renderWithProviders(<ApplicationHelp />, { route: "/characters-explorer" });

    expect(getByLabelText("Help")).toBeDisabled();
    expect(getByTitle("No help written for this screen yet")).toBeInTheDocument();
  });

  it("opens the current application's help from the rail control", async () => {
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
