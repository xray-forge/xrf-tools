import { describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { renderWithTheme } from "#/testing/render";

import { ChoiceListFormRow } from "./ChoiceListFormRow";

function optionsOf(count: number): Array<{ label: string; value: string }> {
  return Array.from({ length: count }, (_, index) => ({ label: `level_${index}`, value: `levels\\level_${index}` }));
}

function renderRow(count: number, onChange = jest.fn()): RenderResult {
  return renderWithTheme(
    <ChoiceListFormRow
      label={"Level"}
      options={optionsOf(count)}
      value={`levels\\level_0`}
      onChange={onChange as (value: string) => void}
    />
  );
}

describe("ChoiceListFormRow", () => {
  it("lists every option as a selectable row", () => {
    const { getAllByRole } = renderRow(3);

    expect(getAllByRole("option")).toHaveLength(3);
    expect(getAllByRole("option")[0].getAttribute("aria-selected")).toBe("true");
  });

  // A toggle group overflows its row past about half a dozen; the whole point of this control is the ones past the
  // edge stay reachable.
  it("offers a filter once there are more options than a row can show", () => {
    expect(renderRow(3).queryByLabelText("Filter level")).toBeNull();
    expect(renderRow(30).queryByLabelText("Filter level")).not.toBeNull();
  });

  // The hint says how the filter matches, whatever the list holds.
  it("hints that the filter matches the options' names", () => {
    expect(renderRow(30).getByLabelText("Filter level")).toHaveAttribute("placeholder", "Filter by name");
  });

  it("narrows to what the filter matches", async () => {
    const { getByLabelText, getAllByRole } = renderRow(30);

    await userEvent.type(getByLabelText("Filter level"), "level_1");

    // `level_1` and `level_10` through `level_19`.
    expect(getAllByRole("option")).toHaveLength(11);
  });

  // An empty list under a filter reads as a broken one.
  it("says so when a filter matches nothing", async () => {
    const { getByLabelText, queryAllByRole, getByText } = renderRow(30);

    await userEvent.type(getByLabelText("Filter level"), "zaton");

    expect(queryAllByRole("option")).toHaveLength(0);
    expect(getByText("Nothing here matches 'zaton'")).toBeTruthy();
  });

  it("lists neighbours sharing a section under one heading, and says what the option chosen is", () => {
    const { getByRole, getByText, queryByText } = renderWithTheme(
      <ChoiceListFormRow
        label={"Shading"}
        options={[
          { label: "Final", value: "final", section: "Frame", description: "The finished frame." },
          { label: "Clay", value: "clay", section: "Frame", description: "One grey for everything." },
          { label: "Depth", value: "depth", section: "Surfaces", description: "Near dark, far light." },
        ]}
        value={"clay"}
        onChange={jest.fn()}
      />
    );

    expect(getByRole("group", { name: "Frame" }).querySelectorAll("[role=option]")).toHaveLength(2);
    expect(getByRole("group", { name: "Surfaces" }).querySelectorAll("[role=option]")).toHaveLength(1);
    expect(getByText("One grey for everything.")).toBeTruthy();
    expect(queryByText("The finished frame.")).toBeNull();
  });

  it("answers with the value of the row chosen", async () => {
    const onChange = jest.fn();
    const { getAllByRole } = renderRow(4, onChange);

    await userEvent.click(getAllByRole("option")[2]);

    expect(onChange).toHaveBeenCalledWith("levels\\level_2");
  });
});
