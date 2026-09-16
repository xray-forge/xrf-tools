import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveLevelEnvModDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelEnvModDescription, mockArchiveLevelEnvModifier } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelEnvModView } from "./ArchiveLevelEnvModView";

function renderView(description: ArchiveLevelEnvModDescription = mockArchiveLevelEnvModDescription()): RenderResult {
  return renderWithProviders(<ArchiveLevelEnvModView description={description} />);
}

describe("ArchiveLevelEnvModView", () => {
  it("gives each override a section of its own, because one is five values rather than a row", () => {
    const { getByText } = renderView();

    expect(getByText("Override 1")).toBeTruthy();
    expect(getByText("40.0 m")).toBeTruthy();
    expect(getByText("1.00")).toBeTruthy();
    expect(getByText("0.850")).toBeTruthy();
  });

  it("names the values the override mixes into, rather than the flag word they came from", () => {
    const { getByText } = renderView();

    expect(getByText("fog colour, fog density")).toBeTruthy();
    expect(getByText("The values the file says to mix in")).toBeTruthy();
  });

  it("says when a file carries no flag word and the engine therefore mixes in everything", () => {
    // Below version 0x0016 there is no word to read, so the reader hands over every parameter named.
    const { getByText } = renderView(
      mockArchiveLevelEnvModDescription({
        version: 21,
        modifiers: [
          mockArchiveLevelEnvModifier({
            declaresParameters: false,
            usedParameters: [
              "view distance",
              "fog colour",
              "fog density",
              "ambient colour",
              "sky colour",
              "hemi colour",
            ],
          }),
        ],
      })
    );

    expect(getByText(/carries no flag word/)).toBeTruthy();
  });

  it("says plainly that a level with no override is the weather cycle everywhere", () => {
    // Most shipped levels are this, so the empty case is the common one rather than an edge.
    const { getByText, queryByText } = renderView(mockArchiveLevelEnvModDescription({ modifiers: [] }));

    expect(getByText("The weather cycle stands everywhere on this level")).toBeTruthy();
    expect(queryByText("Override 1")).toBeNull();
  });
});
