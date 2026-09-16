import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveDescribeScope, ArchiveLevelFogVolDescription, EArchiveDescribeScope } from "@/core/ipc/types/xrf-app";
import { mockArchiveLevelFogVolDescription } from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelFogVolView } from "./ArchiveLevelFogVolView";

const SCOPE: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(description: ArchiveLevelFogVolDescription = mockArchiveLevelFogVolDescription()): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveLevelFogVolView description={description} scope={SCOPE} />, { container });
}

describe("ArchiveLevelFogVolView", () => {
  it("offers the LTX a body reads its simulation from, which is not in this file", () => {
    const { getByText } = renderView();

    expect(getByText("fog_vol_default").closest("button")).not.toBeNull();
    expect(getByText(/live in the LTX it names/)).toBeTruthy();
  });

  it("puts a body's obstacles beside where its profile resolved", () => {
    const { getByText } = renderView();

    expect(getByText("144 obstacles · configs\\environment\\fog_vol_default.ltx")).toBeTruthy();
  });

  it("says plainly that a file was written and nothing was placed in it", () => {
    // Every shipped level but one is this, so the empty case is what the view is mostly asked to draw.
    const { getByText, queryByText } = renderView(mockArchiveLevelFogVolDescription({ volumes: [], obstacles: 0 }));

    expect(getByText("The file was written, and nothing was placed in it")).toBeTruthy();
    expect(queryByText("Body 1")).toBeNull();
  });
});
