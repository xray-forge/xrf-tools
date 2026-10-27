import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent, render } from "@testing-library/react";

import { ColorPicker } from "./ColorPicker";

import { IPickedColor } from "#/color/color-intensity";

describe("ColorPicker", () => {
  // A drag moves the colour every frame; its owner writes the weather and its memory, so it is told at most an interval.
  it("tells a colour moved quickly once at first, then the last once it is let go", () => {
    const onChange = jest.fn<(value: IPickedColor) => void>();
    const { getByRole } = render(<ColorPicker value={{ b: 0, g: 0, r: 255 }} onChange={onChange} />);
    const hue: HTMLElement = getByRole("slider", { name: "Hue" });

    fireEvent.keyDown(hue, { key: "ArrowRight", keyCode: 39 });
    fireEvent.keyDown(hue, { key: "ArrowRight", keyCode: 39 });
    fireEvent.keyDown(hue, { key: "ArrowRight", keyCode: 39 });

    expect(onChange).toHaveBeenCalledTimes(1);

    fireEvent.keyUp(hue, { key: "ArrowRight", keyCode: 39 });

    expect(onChange).toHaveBeenCalledTimes(2);
    expect(onChange.mock.calls[1]?.[0]).not.toEqual(onChange.mock.calls[0]?.[0]);
  });

  // The picker tracks a drag across the window, so the pointer can be let go anywhere; the owner's colour then shows.
  it("shows its owner's colour again once a drag is let go outside it", () => {
    const onChange = jest.fn<(value: IPickedColor) => void>();
    const { getByRole } = render(<ColorPicker value={{ b: 0, g: 0, r: 255 }} onChange={onChange} />);
    const hue: HTMLElement = getByRole("slider", { name: "Hue" });

    fireEvent.keyDown(hue, { key: "ArrowRight", keyCode: 39 });

    expect(hue).not.toHaveAttribute("aria-valuenow", "0");

    fireEvent.pointerUp(window);

    expect(hue).toHaveAttribute("aria-valuenow", "0");
  });
});
