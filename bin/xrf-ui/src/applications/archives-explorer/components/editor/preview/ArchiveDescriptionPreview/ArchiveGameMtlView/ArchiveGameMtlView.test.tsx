import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchiveGameMtlDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveGameMtlDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveGameMtlView } from "./ArchiveGameMtlView";

function renderView(description: ArchiveGameMtlDescription = mockArchiveGameMtlDescription()): RenderResult {
  return renderWithProviders(<ArchiveGameMtlView description={description} />);
}

describe("ArchiveGameMtlView", () => {
  it("counts the pairings rather than listing them", () => {
    // A library carries up to 1,550 pairings; what a reader asks is how many declare anything of their own.
    const { getByText, queryByText } = renderView();

    expect(getByText("1,421")).toBeTruthy();
    expect(getByText(/968 take their behaviour from another pairing/)).toBeTruthy();
    expect(queryByText("default ↔ materials\\acid")).toBeNull();
  });

  it("says how many pairings declare each thing a pairing can declare", () => {
    const { getByText } = renderView();

    expect(getByText("Step sounds")).toBeTruthy();
    expect(getByText("240")).toBeTruthy();
  });

  it("names a material as what a collision face stores by number", () => {
    const { getByText } = renderView();

    expect(getByText("default")).toBeTruthy();
    expect(getByText(/stores one of these by number/)).toBeTruthy();
  });

  it("gives each material the figures that decide what meets it", () => {
    const { getByText } = renderView();

    expect(getByText("friction 1.00 · bounce 0.10")).toBeTruthy();
    expect(getByText("shoot 0.50 · sound 0.25")).toBeTruthy();
  });

  it("folds a description, the flags and the cost of standing in it onto one line", () => {
    const { getByText } = renderView();

    expect(getByText("everything not named otherwise · skidmark, shootable")).toBeTruthy();
    expect(getByText(/liquid, injurious · costs 0.35 health a second to stand in/)).toBeTruthy();
  });

  it("filters the materials by name, because a library of 65 is looked up rather than read", () => {
    const { getByText, getByLabelText, queryByText } = renderView();

    fireEvent.change(getByLabelText("Filter materials"), { target: { value: "acid" } });

    expect(getByText("Materials (1 of 2)")).toBeTruthy();
    expect(queryByText("default")).toBeNull();
  });

  it("says plainly when a filter matches no material", () => {
    const { getByText, getByLabelText } = renderView();

    fireEvent.change(getByLabelText("Filter materials"), { target: { value: "granite" } });

    expect(getByText("No material of this library is named that.")).toBeTruthy();
  });
});
