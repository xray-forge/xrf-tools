import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  ArchiveDescribeScope,
  ArchiveLevelSndStaticDescription,
  EArchiveDescribeScope,
} from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelSndStaticDescription } from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelSndStaticView } from "./ArchiveLevelSndStaticView";

const SCOPE: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(
  description: ArchiveLevelSndStaticDescription = mockArchiveLevelSndStaticDescription()
): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveLevelSndStaticView description={description} scope={SCOPE} />, { container });
}

describe("ArchiveLevelSndStaticView", () => {
  it("leads with what plays without anything asking for it", () => {
    const { getByText } = renderView();

    expect(getByText("Sounds (3)")).toBeTruthy();
    expect(getByText(/without an object asking for it/)).toBeTruthy();
  });

  it("offers a resolved sound as somewhere to go and an absent one as plain text", () => {
    const { getByText } = renderView();

    expect(getByText("ambient\\day\\birds_1").closest("button")).not.toBeNull();
    expect(getByText("ambient\\night\\owl_2").closest("button")).toBeNull();
  });

  it("writes a window of the day as a clock reads it", () => {
    // The pair is whole hours the engine compares against, so rendering them as raw numbers would read as an index.
    const { getByText } = renderView();

    expect(getByText("Between 22:00 and 04:00")).toBeTruthy();
  });

  it("does not present a pair of zeroes as a window of no length", () => {
    const { queryByText } = renderView();

    expect(queryByText("Between 00:00 and 00:00")).toBeNull();
  });

  it("says a record naming no sound at all rather than drawing an empty name", () => {
    const { getByText } = renderView();

    expect(getByText("Names no sound")).toBeTruthy();
  });

  it("gives each sound its own volume and pitch, which is what distinguishes two plantings of one file", () => {
    const { getAllByText, getByText } = renderView();

    expect(getAllByText("volume 0.70 · frequency 1.00")).toHaveLength(2);
    expect(getByText("volume 0.35 · frequency 1.25")).toBeTruthy();
  });
});
