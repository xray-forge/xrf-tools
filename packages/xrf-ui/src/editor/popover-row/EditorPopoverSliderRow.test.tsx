import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverSliderRow } from "./EditorPopoverSliderRow";

function formatMetres(value: number): string {
  return `${value} m`;
}

describe("EditorPopoverSliderRow", () => {
  it("names the slider by its label, describes it by what it does and where it comes from, and reads it formatted", () => {
    const { getByRole, getByText } = renderWithTheme(
      <EditorPopoverSliderRow
        label={"Draw distance"}
        description={"How far the world is drawn."}
        hint={"r__far_plane"}
        value={450}
        min={50}
        max={1500}
        step={10}
        format={formatMetres}
        onChange={jest.fn()}
      />
    );
    const slider: HTMLElement = getByRole("slider", { name: "Draw distance" });

    expect(slider).toHaveAccessibleDescription("How far the world is drawn. r__far_plane");
    expect(slider).toHaveAttribute("aria-valuetext", "450 m");
    expect(getByText("450 m")).toBeInTheDocument();
  });

  it("tells a step taken by a key at once", () => {
    const onChange = jest.fn();
    const { getByRole } = renderWithTheme(
      <EditorPopoverSliderRow
        label={"Exposure"}
        value={2}
        min={0}
        max={10}
        step={1}
        format={String}
        onChange={onChange}
      />
    );

    fireEvent.keyDown(getByRole("slider", { name: "Exposure" }), { key: "ArrowRight" });

    expect(onChange).toHaveBeenCalledWith(3);
  });

  it("blocks a disabled slider", () => {
    const { getByRole } = renderWithTheme(
      <EditorPopoverSliderRow
        label={"Exposure"}
        value={2}
        min={0}
        max={10}
        step={1}
        format={String}
        isDisabled
        onChange={jest.fn()}
      />
    );

    expect(getByRole("slider", { name: "Exposure" })).toBeDisabled();
  });
});
