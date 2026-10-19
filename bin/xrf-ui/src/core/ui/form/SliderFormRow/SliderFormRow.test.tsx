import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import {
  describeSliderRecommendation,
  ISliderReferenceSource,
  SLIDER_ABOVE_RECOMMENDED,
  SliderReferenceContext,
} from "@/core/ui/form/ReferenceSlider";
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

describe("SliderFormRow against its reference", () => {
  const source: ISliderReferenceSource = { label: "Base preset", resolve: () => 32 };

  it("marks the reference, offers a reset to it once left, and warns outside the recommended range", async () => {
    const onChange = jest.fn();
    const { getByRole, queryByRole, rerender } = renderWithProviders(
      <SliderReferenceContext.Provider value={source}>
        <SliderFormRow
          label={"Steps"}
          value={32}
          min={16}
          max={128}
          step={8}
          referencePath={["reflections", "steps"]}
          recommended={[32, 96]}
          onChange={onChange}
        />
      </SliderReferenceContext.Provider>
    );

    expect(queryByRole("button", { name: "Reset to Base preset: 32" })).toBeNull();

    rerender(
      <SliderReferenceContext.Provider value={source}>
        <SliderFormRow
          label={"Steps"}
          value={128}
          min={16}
          max={128}
          step={8}
          referencePath={["reflections", "steps"]}
          recommended={[32, 96]}
          onChange={onChange}
        />
      </SliderReferenceContext.Provider>
    );

    expect(describeSliderRecommendation(128, [32, 96])).toBe(SLIDER_ABOVE_RECOMMENDED);

    await userEvent.click(getByRole("button", { name: "Reset to Base preset: 32" }));

    expect(onChange).toHaveBeenCalledWith(32);
  });
});
