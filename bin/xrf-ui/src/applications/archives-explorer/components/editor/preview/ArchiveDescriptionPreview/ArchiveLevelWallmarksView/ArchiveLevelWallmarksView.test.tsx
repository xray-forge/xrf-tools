import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  ArchiveDescribeScope,
  ArchiveLevelWallmarksDescription,
  EArchiveDescribeScope,
} from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelWallmarksDescription } from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelWallmarksView } from "./ArchiveLevelWallmarksView";

const SCOPE: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(
  description: ArchiveLevelWallmarksDescription = mockArchiveLevelWallmarksDescription()
): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveLevelWallmarksView description={description} scope={SCOPE} />, { container });
}

describe("ArchiveLevelWallmarksView", () => {
  it("leads with the decals and what they cost to draw", () => {
    const { getByText } = renderView();

    expect(getByText("441")).toBeTruthy();
    expect(getByText("Over 5,292 vertices, which is what the layer costs to draw")).toBeTruthy();
  });

  it("says outright that the runtime reads none of this", () => {
    // The editor bakes the file; CWallmarksEngine places its own at play time, so a reader should not assume otherwise.
    const { getByText } = renderView();

    expect(getByText("Nothing at play time")).toBeTruthy();
    expect(getByText(/places its own wallmarks/)).toBeTruthy();
  });

  it("offers the texture a slot draws with and names the blender beside it", () => {
    const { getByText } = renderView();

    expect(getByText("wm\\wm_blood").closest("button")).not.toBeNull();
    expect(getByText("effects\\wallmark · 4,164 vertices")).toBeTruthy();
  });

  it("says a slot naming no texture rather than drawing an empty name", () => {
    const { getByText } = renderView();

    expect(getByText("Names no texture")).toBeTruthy();
  });
});
