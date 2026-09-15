import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveSpawnDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveSpawnDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveSpawnDescriptionView } from "./ArchiveSpawnDescriptionView";

function renderView(description: ArchiveSpawnDescription = mockArchiveSpawnDescription()): RenderResult {
  return renderWithProviders(<ArchiveSpawnDescriptionView description={description} />);
}

describe("ArchiveSpawnDescriptionView", () => {
  it("leads with the counts the header carries", () => {
    const { getByText } = renderView();

    expect(getByText("6464")).toBeTruthy();
    expect(getByText("Across 5 levels")).toBeTruthy();
  });

  it("names both identities, which are different things", () => {
    const { getByText } = renderView();

    expect(getByText("What a save game is pinned to")).toBeTruthy();
    expect(getByText("The graph this set was built against")).toBeTruthy();
  });

  it("says where the weight of the file sits", () => {
    // Vanilla's game graph is 88% of a 29 MB set, which is the whole reason the sections are listed.
    const { getByText } = renderView();

    expect(getByText("Game graph")).toBeTruthy();
    expect(getByText("89% of the file")).toBeTruthy();
  });

  it("leaves a section too small to matter unqualified", () => {
    const { queryByText } = renderView();

    expect(queryByText("0% of the file")).toBeNull();
  });

  it("falls back to the id for a section the format does not name", () => {
    const { getByText } = renderView();

    expect(getByText("0x0009")).toBeTruthy();
  });
});
