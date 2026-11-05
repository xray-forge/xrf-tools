import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverTextRow } from "./EditorPopoverTextRow";

describe("EditorPopoverTextRow", () => {
  it("names the field by its label, describes it, and hands back what is typed", () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPopoverTextRow
        label={"Bump texture"}
        description={"Taken verbatim"}
        value={"ston\\ston_bump"}
        onChange={onChange}
      />
    );
    const field: HTMLElement = getByRole("textbox", { name: "Bump texture" });

    expect(field).toHaveValue("ston\\ston_bump");
    expect(field).toHaveAccessibleDescription("Taken verbatim");

    fireEvent.change(field, { target: { value: "ston\\other_bump" } });

    expect(onChange).toHaveBeenCalledWith("ston\\other_bump");
  });

  it("leaves a half-typed number alone that reads as the value it already holds", () => {
    const { getByRole } = renderWithTheme(
      <EditorPopoverTextRow label={"Scale"} value={0} type={"number"} onChange={jest.fn()} />
    );
    const field: HTMLInputElement = getByRole("spinbutton", { name: "Scale" }) as HTMLInputElement;

    fireEvent.change(field, { target: { value: "0.0" } });

    expect(field.value).toBe("0.0");
  });

  it("shows a value it cannot change read-only", () => {
    const { getByRole } = renderWithTheme(<EditorPopoverTextRow label={"Width"} value={"512"} type={"number"} />);

    expect(getByRole("spinbutton", { name: "Width" })).toHaveAttribute("readonly");
  });
});
