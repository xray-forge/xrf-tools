import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchiveLightAnimDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLightAnimDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLightAnimView } from "./ArchiveLightAnimView";

function renderView(description: ArchiveLightAnimDescription = mockArchiveLightAnimDescription()): RenderResult {
  return renderWithProviders(<ArchiveLightAnimView description={description} />);
}

describe("ArchiveLightAnimView", () => {
  it("leads with the animations and the keys across them", () => {
    const { getByText } = renderView();

    expect(getByText("Over 620 keyed colours")).toBeTruthy();
    expect(getByText("Animations (3)")).toBeTruthy();
  });

  it("gives each animation its length with the rate behind it", () => {
    const { getByText } = renderView();

    expect(getByText("2.00 s · 30 frames at 15 fps")).toBeTruthy();
    expect(getByText("2.00 s · 60 frames at 30 fps")).toBeTruthy();
  });

  it("says an animation never advances rather than reporting a length of nothing", () => {
    // A rate of zero has no duration to divide out, and rendering one would state a measurement never taken.
    const { getByText } = renderView();

    expect(getByText("1 frames · never advances")).toBeTruthy();
  });

  it("says nothing about channel order at the version that stores colours as the engine uses them", () => {
    const { getByText } = renderView();

    expect(getByText("Colours are stored as the engine uses them")).toBeTruthy();
  });

  it("warns that an older library stores its colours swapped", () => {
    const { getByText } = renderView(mockArchiveLightAnimDescription({ version: 0, isBgr: true }));

    expect(getByText(/stored channel-swapped at this version/)).toBeTruthy();
  });

  it("filters by name, because a library of 157 is looked up rather than read", () => {
    const { getByText, getByLabelText } = renderView();

    fireEvent.change(getByLabelText("Filter animations"), { target: { value: "lamp" } });

    expect(getByText("Animations (1 of 3)")).toBeTruthy();
  });
});
