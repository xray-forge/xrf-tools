import { describe, expect, it, jest } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { EditorPopoverVectorRow } from "./EditorPopoverVectorRow";

describe("EditorPopoverVectorRow", () => {
  it("names each component and tells the vector once one is let go", async () => {
    const onChange = jest.fn<(value: Array<number>) => void>();
    const { getByRole } = renderWithTheme(
      <EditorPopoverVectorRow label={"wind"} description={"wind_direction"} value={[1, 0, 0.5]} onChange={onChange} />
    );
    const y: HTMLElement = getByRole("textbox", { name: "wind y" });

    expect(getByRole("group", { name: "wind" })).toHaveAccessibleDescription("wind_direction");

    await userEvent.clear(y);
    await userEvent.type(y, "2");

    expect(onChange).not.toHaveBeenCalled();

    await userEvent.type(y, "{Enter}");

    expect(onChange).toHaveBeenLastCalledWith([1, 2, 0.5]);
  });

  // A colour brighter than one, which no picker holds: the intensity scales the colour past it.
  it("types a colour's intensity, which scales its channels and keeps its alpha", async () => {
    const onChange = jest.fn<(value: Array<number>) => void>();
    const { getByRole } = renderWithTheme(
      <EditorPopoverVectorRow label={"sun_color"} value={[2, 1, 0.5, 1]} isColor onChange={onChange} />
    );
    const intensity: HTMLElement = getByRole("textbox", { name: "sun_color intensity" });

    expect(intensity).toHaveValue("2");

    await userEvent.clear(intensity);
    await userEvent.type(intensity, "4{Enter}");

    expect(onChange).toHaveBeenLastCalledWith([4, 2, 1, 1]);
  });

  it("opens one colour's picker at a time, inline under its numbers", async () => {
    const { getByRole, queryAllByTestId } = renderWithTheme(
      <>
        <EditorPopoverVectorRow label={"sun_color"} value={[1, 1, 1]} isColor onChange={() => {}} />
        <EditorPopoverVectorRow label={"fog_color"} value={[0.5, 0.5, 0.5]} isColor onChange={() => {}} />
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
