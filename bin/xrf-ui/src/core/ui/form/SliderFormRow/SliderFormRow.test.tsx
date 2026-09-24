import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";

import { renderWithProviders } from "@/fixtures/utils/render";

import { SliderFormRow } from "./SliderFormRow";

describe("SliderFormRow", () => {
  it("names the slider by its label, reads its value out, and reports a new one", () => {
    const onChange = jest.fn();
    const { getByRole, getByText } = renderWithProviders(
      <SliderFormRow
        label={"Distance"}
        value={1}
        min={0}
        max={3}
        step={0.5}
        format={(value: number) => `${value}x`}
        onChange={onChange}
      />
    );
    const slider: HTMLElement = getByRole("slider", { name: "Distance" });

    expect(getByText("1x")).toBeInTheDocument();
    expect(slider).toHaveAttribute("aria-valuetext", "1x");

    fireEvent.change(slider, { target: { value: "2" } });

    expect(onChange).toHaveBeenCalledWith(2);
  });
});
