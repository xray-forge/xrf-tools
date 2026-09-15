import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  ArchiveDescribeScope,
  ArchiveLevelDescription,
  EArchiveDescribeScope,
  EArchiveLevelEntry,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import {
  mockArchiveLevelDescription,
  mockArchiveLevelSurface,
  mockArchiveReference,
} from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveLevelDescriptionView } from "./ArchiveLevelDescriptionView";

const VOLUMES: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(description: ArchiveLevelDescription = mockArchiveLevelDescription()): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveLevelDescriptionView description={description} scope={VOLUMES} />, { container });
}

describe("ArchiveLevelDescriptionView", () => {
  it("leads with what the bundle holds and what built it", () => {
    const { getByText } = renderView();

    expect(getByText("xrLC 14")).toBeTruthy();
    expect(getByText("Quality 2")).toBeTruthy();
    expect(getByText("Surfaces (2)")).toBeTruthy();
  });

  it("addresses a surface by its position, which is what a face names", () => {
    const { getByText } = renderView();

    expect(getByText("#1")).toBeTruthy();
    expect(getByText("default")).toBeTruthy();
  });

  it("explains the empty row every built level carries rather than hiding it", () => {
    const { getByText } = renderView();

    expect(getByText("Empty")).toBeTruthy();
    expect(getByText(/renderer skips. Every built level has one/)).toBeTruthy();
  });

  it("offers the shader library it asked, so a name nothing defines can be read there", () => {
    const { getByRole } = renderView();

    expect(getByRole("button", { name: "shaders.xr" })).toBeTruthy();
  });

  it("says a shader name was carried unasked when no library is open, rather than calling it absent", () => {
    const { getByText, queryByText } = renderView(
      mockArchiveLevelDescription({
        surfaces: [
          mockArchiveLevelSurface(0, {
            index: 0,
            entry: {
              kind: EArchiveLevelEntry.DRAWN,
              shader: { name: "flora\\leaf_wave", status: EArchiveReferenceStatus.UNKNOWN },
              textures: [],
            },
          }),
        ],
        bundle: {
          ...mockArchiveLevelDescription().bundle,
          library: null,
          surfaces: 1,
        },
      })
    );

    expect(getByText(/No shader library in these volumes to ask/)).toBeTruthy();
    expect(queryByText(/Not defined by the shader library/)).toBeNull();
    expect(getByText(/open the tree holding shaders.xr/)).toBeTruthy();
  });

  it("says what the renderer does with a row it cannot split", () => {
    const { getByText } = renderView(
      mockArchiveLevelDescription({
        surfaces: [
          mockArchiveLevelSurface(4, { index: 4, entry: { kind: EArchiveLevelEntry.UNUSABLE, raw: "brokenentry" } }),
        ],
      })
    );

    expect(getByText("brokenentry")).toBeTruthy();
    expect(getByText(/dereferences the result without checking it/)).toBeTruthy();
  });

  it("filters by a texture as well as by a shader, which are the two ends of one question", () => {
    const { getByLabelText, getByText } = renderView(
      mockArchiveLevelDescription({
        surfaces: [
          mockArchiveLevelSurface(1),
          mockArchiveLevelSurface(2, {
            index: 2,
            entry: {
              kind: EArchiveLevelEntry.DRAWN,
              shader: { name: "flora\\leaf_wave", status: EArchiveReferenceStatus.PRESENT },
              textures: [mockArchiveReference({ name: "flora\\leaf_oak", path: null, entry: null })],
            },
          }),
        ],
      })
    );

    fireEvent.change(getByLabelText("Filter by shader or texture"), { target: { value: "leaf_oak" } });

    expect(getByText("Surfaces (1 of 2)")).toBeTruthy();
    expect(getByText("flora\\leaf_wave")).toBeTruthy();
  });
});
