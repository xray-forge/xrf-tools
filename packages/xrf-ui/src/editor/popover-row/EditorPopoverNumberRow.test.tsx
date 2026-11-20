import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverNumberRow } from "./EditorPopoverNumberRow";

describe("EditorPopoverNumberRow", () => {
  it("names the field by its label and tells each number typed as a number", () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPopoverNumberRow label={"Bump height"} description={"Parallax depth"} value={0.5} onChange={onChange} />
    );
    const field: HTMLElement = getByRole("spinbutton", { name: "Bump height" });

    expect(field).toHaveValue(0.5);
    expect(field).toHaveAccessibleDescription("Parallax depth");

    fireEvent.change(field, { target: { value: "-1.25" } });

    expect(onChange).toHaveBeenCalledWith(-1.25);
  });

  // A field mid-edit would otherwise write zero for a cleared field into the owner's value.
  it("tells nothing for a field cleared to be typed again", () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithTheme(<EditorPopoverNumberRow label={"Scale"} value={2} onChange={onChange} />);

    const field: HTMLElement = getByRole("spinbutton", { name: "Scale" });

    fireEvent.change(field, { target: { value: "" } });

    expect(onChange).not.toHaveBeenCalled();
    // The owner's number is not written back over what the hand typed.
    expect(field).toHaveValue(null);
  });

  it("takes a value set from elsewhere over what was typed", () => {
    const { getByRole, rerender } = renderWithTheme(
      <EditorPopoverNumberRow label={"Scale"} value={2} onChange={jest.fn()} />
    );
    const field: HTMLElement = getByRole("spinbutton", { name: "Scale" });

    fireEvent.change(field, { target: { value: "" } });
    rerender(<EditorPopoverNumberRow label={"Scale"} value={3} onChange={jest.fn()} />);

    expect(field).toHaveValue(3);
  });

  it("shows the owner's value again once an edit it refused or left empty loses focus", () => {
    // The owner keeps 2 whatever is typed, as one clamping to a range would.
    const { getByRole } = renderWithTheme(<EditorPopoverNumberRow label={"Scale"} value={2} onChange={jest.fn()} />);
    const field: HTMLElement = getByRole("spinbutton", { name: "Scale" });

    fireEvent.change(field, { target: { value: "9" } });

    expect(field).toHaveValue(9);

    fireEvent.blur(field);

    expect(field).toHaveValue(2);

    fireEvent.change(field, { target: { value: "" } });
    fireEvent.blur(field);

    expect(field).toHaveValue(2);
  });

  it("shows a value there is no number for empty, and one it cannot change read-only", () => {
    const { getByRole } = renderWithTheme(<EditorPopoverNumberRow label={"Fade"} value={null} />);
    const field: HTMLElement = getByRole("spinbutton", { name: "Fade" });

    expect(field).toHaveValue(null);
    expect(field).toHaveAttribute("readonly");
  });
});
