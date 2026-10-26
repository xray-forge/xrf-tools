import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveHeaderEntries } from "./ArchiveHeaderEntries";

const HEADER: string = "[header]\r\nauto_load = true\r\nentry_point = gamedata\r\nauthor = Original\r\n";

describe("ArchiveHeaderEntries", () => {
  it("edits and removes custom values while preserving the dedicated header settings", async () => {
    const onChange = jest.fn();
    const { getByRole, queryByRole } = renderWithProviders(
      <ArchiveHeaderEntries header={HEADER} onChange={onChange} />
    );

    expect(queryByRole("textbox", { name: "Value of auto_load" })).not.toBeInTheDocument();
    expect(queryByRole("textbox", { name: "Value of entry_point" })).not.toBeInTheDocument();

    fireEvent.change(getByRole("textbox", { name: "Value of author" }), { target: { value: "Updated" } });

    expect(onChange).toHaveBeenLastCalledWith(
      "[header]\r\nauto_load = true\r\nentry_point = gamedata\r\nauthor = Updated\r\n"
    );

    await userEvent.click(getByRole("button", { name: "Remove author" }));

    expect(onChange).toHaveBeenLastCalledWith("[header]\r\nauto_load = true\r\nentry_point = gamedata\r\n");
  });

  it.each(["author", "auto_load"])("rejects the existing key %s even with surrounding whitespace", async (key) => {
    const onChange = jest.fn();
    const { getByRole } = renderWithProviders(<ArchiveHeaderEntries header={HEADER} onChange={onChange} />);

    fireEvent.change(getByRole("textbox", { name: "New header key" }), { target: { value: ` ${key} ` } });
    fireEvent.change(getByRole("textbox", { name: "New header value" }), { target: { value: "Replacement" } });

    const button: HTMLElement = getByRole("button", { name: "Add header value" });

    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("That key is already in the header");
    expect(getByRole("textbox", { name: "New header key" })).toHaveAttribute("aria-invalid", "true");

    fireEvent.click(button);

    expect(onChange).not.toHaveBeenCalled();
  });

  it("requires a key and clears the draft after adding a new value", async () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithProviders(<ArchiveHeaderEntries header={null} onChange={onChange} />);
    const button: HTMLElement = getByRole("button", { name: "Add header value" });

    expect(button).toBeDisabled();
    expect(button).toHaveAccessibleDescription("Enter a header key");

    fireEvent.change(getByRole("textbox", { name: "New header key" }), { target: { value: " author " } });
    fireEvent.change(getByRole("textbox", { name: "New header value" }), { target: { value: " XRF " } });
    await userEvent.click(button);

    expect(onChange).toHaveBeenCalledWith("[header]\r\nauthor = XRF\r\n");
    expect(getByRole("textbox", { name: "New header key" })).toHaveValue("");
    expect(getByRole("textbox", { name: "New header value" })).toHaveValue("");
    expect(button).toBeDisabled();
  });

  it("removes the header when the last value is cleared", () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithProviders(
      <ArchiveHeaderEntries header={"[header]\r\nauthor = XRF\r\n"} onChange={onChange} />
    );

    fireEvent.change(getByRole("textbox", { name: "Value of author" }), { target: { value: "" } });

    expect(onChange).toHaveBeenCalledWith(null);
  });

  it("disables both existing entries and the draft while an operation is running", () => {
    const onChange = jest.fn();
    const { getByRole, getAllByRole } = renderWithProviders(
      <ArchiveHeaderEntries header={HEADER} isDisabled={true} onChange={onChange} />
    );

    for (const input of getAllByRole("textbox")) {
      expect(input).toBeDisabled();
    }

    expect(getByRole("button", { name: "Remove author" })).toBeDisabled();
    expect(getByRole("button", { name: "Add header value" })).toBeDisabled();
    expect(getByRole("button", { name: "Add header value" })).toHaveAccessibleDescription(
      "Wait for the current operation to finish before editing"
    );
  });
});
