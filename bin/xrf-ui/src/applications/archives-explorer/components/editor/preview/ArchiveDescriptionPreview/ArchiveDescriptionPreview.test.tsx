import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveFileDescription } from "@/core/ipc/types/xrf-app";
import {
  mockArchiveFileDescription,
  mockArchiveReference,
  mockArchiveThmDescription,
} from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { AsyncState } from "@/lib/async-state";

import { ArchiveDescriptionPreview } from "./ArchiveDescriptionPreview";

function renderPreview(description: ArchiveFileDescription): RenderResult {
  const { service, container } = mockInjectedService(ArchivesService);

  service.content = AsyncState.ready({ kind: "description" as const, description });

  return renderWithProviders(<ArchiveDescriptionPreview />, { container });
}

describe("ArchiveDescriptionPreview", () => {
  it("leads with the texture the descriptor describes", () => {
    const { getByText } = renderPreview(mockArchiveFileDescription());

    expect(getByText("512 × 512 · DXT5 · 10 mips")).toBeTruthy();
    expect(getByText("act\\act_arm_1")).toBeTruthy();
  });

  it("explains a cube map's declared strip rather than presenting it as a disagreement", () => {
    // 53 of vanilla's 54 declared-size differences are this, so a surface that cannot say it is noise.
    const { getByText } = renderPreview(
      mockArchiveFileDescription({
        kind: "thm",
        description: mockArchiveThmDescription({
          texture: {
            reference: mockArchiveReference(),
            shape: { width: 512, height: 512, mipmapLevels: 1, format: "DXT1" },
            declared: { width: 3072, height: 512, isCubeStrip: true },
          },
          textureType: { label: "Cube Map", value: 1, isReadByEngine: false, isDeclared: true },
        }),
      })
    );

    expect(getByText("3072 × 512")).toBeTruthy();
    expect(getByText(/six cube faces/)).toBeTruthy();
  });

  it("says when the engine reads nothing further than the type", () => {
    const { getByText } = renderPreview(
      mockArchiveFileDescription({
        kind: "thm",
        description: mockArchiveThmDescription({
          textureType: { label: "Bump Map", value: 2, isReadByEngine: false, isDeclared: true },
        }),
      })
    );

    expect(getByText(/LoadTHM takes nothing further from a bump map descriptor/)).toBeTruthy();
  });

  it("words an absent reference as the scope that was searched, without calling it a fault", () => {
    const { getByText, queryByText } = renderPreview(mockArchiveFileDescription());

    expect(getByText("Not in these 3 volumes")).toBeTruthy();
    expect(queryByText(/missing/i)).toBeNull();
  });

  it("words an absence in a world's own terms", () => {
    const { getByText } = renderPreview(
      mockArchiveFileDescription({ kind: "thm", description: mockArchiveThmDescription() }, { kind: "world" })
    );

    expect(getByText("Not found in the mounted tree")).toBeTruthy();
  });

  it("offers a resolved reference as a way out of the description, and an absent one as plain text", () => {
    // What the click then does belongs to the service and is tested there; what this owns is which of the two names
    // is offered as somewhere to go at all.
    const { getByText } = renderPreview(mockArchiveFileDescription());

    expect(getByText("act\\act_arm_1").closest("button")).not.toBeNull();
    expect(getByText("act\\act_arm_1_bump").closest("button")).toBeNull();
  });

  it("distinguishes a chunk the file omits from one holding a default", () => {
    const { getAllByText } = renderPreview(
      mockArchiveFileDescription({
        kind: "thm",
        description: mockArchiveThmDescription({ bump: null, material: null }),
      })
    );

    expect(getAllByText("Not declared")).toHaveLength(2);
  });

  it("names every flag the SDK knows, set or not", () => {
    const { getByText } = renderPreview(mockArchiveFileDescription());

    expect(getByText("flGenerateMipMaps")).toBeTruthy();
    expect(getByText("flDitherColor")).toBeTruthy();
  });

  it("says calmly that a format has no describer yet", () => {
    const { getByText } = renderPreview(
      mockArchiveFileDescription({ kind: "unsupported", reason: { kind: "noDescriber", extension: "omf" } })
    );

    expect(getByText("No description yet")).toBeTruthy();
    expect(getByText(/Nothing reads \.omf files yet/)).toBeTruthy();
  });

  it("reports an entry too large to read whole with both sizes", () => {
    const { getByText } = renderPreview(
      mockArchiveFileDescription({
        kind: "unsupported",
        reason: { kind: "tooLarge", size: 128 * 1024 * 1024, maximum: 64 * 1024 * 1024 },
      })
    );

    expect(getByText(/128 MB, past the 64 MB limit/)).toBeTruthy();
  });
});
