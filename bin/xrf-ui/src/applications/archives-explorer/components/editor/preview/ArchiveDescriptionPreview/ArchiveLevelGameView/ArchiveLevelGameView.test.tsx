import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveLevelGameDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelGameDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelGameView } from "./ArchiveLevelGameView";

function renderView(description: ArchiveLevelGameDescription = mockArchiveLevelGameDescription()): RenderResult {
  return renderWithProviders(<ArchiveLevelGameView description={description} />);
}

describe("ArchiveLevelGameView", () => {
  it("groups thousands of points into the few kinds they are, with the digits readable", () => {
    const { getByText } = renderView();

    expect(getByText("4,951")).toBeTruthy();
    expect(getByText("Across 3 kinds")).toBeTruthy();
    expect(getByText("Actor spawn")).toBeTruthy();
    expect(getByText("4,444")).toBeTruthy();
  });

  it("names the preset only some kinds carry, and leaves the others unqualified", () => {
    const { getByText, queryByText } = renderView();

    expect(getByText("173 of them name a spawn preset")).toBeTruthy();
    expect(queryByText("0 of them name a spawn preset")).toBeNull();
  });

  it("falls back to the stored number for a kind the engine gives no name", () => {
    const { getByText } = renderView(
      mockArchiveLevelGameDescription({ spawns: [{ label: null, kind: 7, points: 4, profiled: 0 }] })
    );

    expect(getByText("Kind 7")).toBeTruthy();
  });

  it("counts the paths and the nodes across them, which is what a path costs to walk", () => {
    const { getByText } = renderView();

    expect(getByText("1,510")).toBeTruthy();
    expect(getByText("Over 8,204 nodes")).toBeTruthy();
  });

  it("names a path carrying nothing to walk, which is a path in name only", () => {
    const { getByText } = renderView();

    expect(getByText("3")).toBeTruthy();
    expect(getByText(/a path in name only/)).toBeTruthy();
  });
});
