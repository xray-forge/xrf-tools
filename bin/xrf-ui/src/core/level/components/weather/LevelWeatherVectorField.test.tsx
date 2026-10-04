import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { renderWithProviders } from "@/fixtures/utils/render";

describe("LevelWeatherVectorField", () => {
  // An engine sun is often brighter than one, which no picker holds: the intensity scales the colour past it.
  it("types a colour's intensity, which scales its channels and keeps its alpha", async () => {
    const onChange = jest.fn<(value: Array<number>) => void>();
    const { getByRole } = renderWithProviders(
      <LevelWeatherVectorField label={"sun_color"} value={[2, 1, 0.5, 1]} isColor onChange={onChange} />
    );
    const intensity: HTMLElement = getByRole("textbox", { name: "sun_color intensity" });

    expect(intensity).toHaveValue("2");

    await userEvent.clear(intensity);
    await userEvent.type(intensity, "4{Enter}");

    expect(onChange).toHaveBeenLastCalledWith([4, 2, 1, 1]);
  });

  it("opens one colour's picker at a time, inline under its numbers", async () => {
    const { getByRole, queryAllByTestId } = renderWithProviders(
      <>
        <LevelWeatherVectorField label={"sun_color"} value={[1, 1, 1]} isColor onChange={() => {}} />
        <LevelWeatherVectorField label={"fog_color"} value={[0.5, 0.5, 0.5]} isColor onChange={() => {}} />
      </>
    );

    await userEvent.click(getByRole("button", { name: "Pick sun_color" }));

    expect(queryAllByTestId("color-picker")).toHaveLength(1);
    expect(getByRole("button", { name: "Pick sun_color" })).toHaveAttribute("aria-expanded", "true");

    await userEvent.click(getByRole("button", { name: "Pick fog_color" }));

    expect(queryAllByTestId("color-picker")).toHaveLength(1);
    expect(getByRole("button", { name: "Pick sun_color" })).toHaveAttribute("aria-expanded", "false");

    await userEvent.click(getByRole("button", { name: "Pick fog_color" }));

    expect(queryAllByTestId("color-picker")).toHaveLength(0);
  });
});
