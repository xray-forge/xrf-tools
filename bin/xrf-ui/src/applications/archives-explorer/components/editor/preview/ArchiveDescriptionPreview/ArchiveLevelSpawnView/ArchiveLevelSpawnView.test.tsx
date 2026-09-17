import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchiveLevelSpawnDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelSpawnDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelSpawnView } from "./ArchiveLevelSpawnView";

function renderView(description: ArchiveLevelSpawnDescription = mockArchiveLevelSpawnDescription()): RenderResult {
  return renderWithProviders(<ArchiveLevelSpawnView description={description} />);
}

describe("ArchiveLevelSpawnView", () => {
  it("leads with what is spawned and out of how many sections", () => {
    const { getByText } = renderView();

    expect(getByText("2,832")).toBeTruthy();
    expect(getByText("Out of 5 sections")).toBeTruthy();
  });

  it("groups thousands of objects into the sections they are built from, most planted first", () => {
    // A level spawns up to 2,832 objects out of at most 175 sections; the sections are what a reader asks for.
    const { getByText } = renderView();

    expect(getByText("Sections (5)")).toBeTruthy();
    expect(getByText("physic_object")).toBeTruthy();
    expect(getByText("1,204 objects")).toBeTruthy();
    expect(getByText("1 object")).toBeTruthy();
  });

  it("says this is the list rather than the set, which is the other format under the extension", () => {
    const { getByText } = renderView();

    expect(getByText("xrServer::SLS_Default")).toBeTruthy();
    expect(getByText(/no header, no graph, no identity/)).toBeTruthy();
  });

  it("says how much of the level the objects stand in", () => {
    const { getByText } = renderView();

    expect(getByText("512.0 × 128.0 × 512.0 m")).toBeTruthy();
  });

  it("does not present a list holding nothing as a box of no size", () => {
    const { getByText } = renderView(mockArchiveLevelSpawnDescription({ objects: 0, sections: [], bounds: null }));

    expect(getByText("Nothing to measure")).toBeTruthy();
  });

  it("filters the sections by name, because a level reaches 175 of them", () => {
    const { getByText, getByLabelText } = renderView();

    fireEvent.change(getByLabelText("Filter sections"), { target: { value: "point" } });

    expect(getByText("Sections (1 of 5)")).toBeTruthy();
  });

  it("says plainly when a filter matches no section", () => {
    const { getByText, getByLabelText } = renderView();

    fireEvent.change(getByLabelText("Filter sections"), { target: { value: "anomaly" } });

    expect(getByText("No section of this level is named that.")).toBeTruthy();
  });
});
