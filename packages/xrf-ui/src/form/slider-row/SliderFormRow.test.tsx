import { describe, expect, it, jest } from "@jest/globals";
import { fireEvent } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { ReactElement } from "react";

import { ISliderReferenceSource, SLIDER_ABOVE_RECOMMENDED, SliderReferenceContext } from "#/form/reference-slider";
import { renderWithTheme } from "#/testing/render";

import { SliderFormRow } from "./SliderFormRow";

const SOURCE: ISliderReferenceSource = { label: "Base preset", resolve: () => 32 };

function renderStepsRow(value: number, onChange: (value: number) => void): ReactElement {
  return (
    <SliderReferenceContext.Provider value={SOURCE}>
      <SliderFormRow
        label={"Steps"}
        value={value}
        min={16}
        max={128}
        step={8}
        referencePath={["reflections", "steps"]}
        recommended={[32, 96]}
        onChange={onChange}
      />
    </SliderReferenceContext.Provider>
  );
}

describe("SliderFormRow", () => {
  it("names the slider by its label, reads its value out, and reports a new one", () => {
    const onChange = jest.fn();
    const { getByRole, getByText } = renderWithTheme(
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

  it("offers a reset to its reference once left, and says why a value outside the recommended range warns", async () => {
    const onChange = jest.fn();
    const { getByRole, getByLabelText, queryByRole, rerender } = renderWithTheme(renderStepsRow(32, onChange));

    expect(queryByRole("button", { name: "Reset to Base preset: 32" })).toBeNull();

    rerender(renderStepsRow(128, onChange));

    expect(getByLabelText(SLIDER_ABOVE_RECOMMENDED)).toHaveTextContent("128");

    await userEvent.click(getByRole("button", { name: "Reset to Base preset: 32" }));

    expect(onChange).toHaveBeenCalledWith(32);
  });
});
