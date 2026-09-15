import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import {
  ArchiveDescribeScope,
  ArchiveShadersDescription,
  EArchiveDescribeScope,
  EArchiveReferenceStatus,
} from "@/core/ipc/types/xrf-app";
import {
  mockArchiveReference,
  mockArchiveShadersBlender,
  mockArchiveShadersDescription,
} from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveShadersDescriptionView } from "./ArchiveShadersDescriptionView";

const VOLUMES: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(description: ArchiveShadersDescription = mockArchiveShadersDescription()): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveShadersDescriptionView description={description} scope={VOLUMES} />, {
    container,
  });
}

describe("ArchiveShadersDescriptionView", () => {
  it("says which of the file's chunks it read, since it is one of four", () => {
    const { getByText } = renderView();

    expect(getByText("Blender chunk only")).toBeTruthy();
    expect(getByText(/name no surface and are skipped/)).toBeTruthy();
  });

  it("heads a blender with the class and version that decide how it is read", () => {
    const { getByText } = renderView();

    expect(getByText("models\\model_aref")).toBeTruthy();
    expect(getByText("MODEL · v2")).toBeTruthy();
  });

  it("lists the property grid rather than counting it", () => {
    const { getByText } = renderView();

    expect(getByText(/Priority.*4 \(0 to 8\)/)).toBeTruthy();
  });

  it("leaves a slot the renderer fills as the name it is, resolving only the properties that name a file", () => {
    const { getByText, queryAllByRole } = renderView();

    expect(getByText(/Name.*\$base0/)).toBeTruthy();
    // Only the one texture property that names a path is selectable.
    expect(queryAllByRole("button")).toHaveLength(1);
  });

  it("words an absent texture as the scope that was searched", () => {
    const { getByText } = renderView(
      mockArchiveShadersDescription({
        blenders: [
          mockArchiveShadersBlender({
            properties: [
              {
                name: "R2-R",
                kind: "Texture",
                value: "detail\\detail_gone",
                texture: mockArchiveReference({
                  name: "detail\\detail_gone",
                  path: "textures\\detail\\detail_gone.dds",
                  entry: null,
                  status: EArchiveReferenceStatus.ABSENT,
                }),
              },
            ],
          }),
        ],
      })
    );

    expect(getByText(/Not in these 3 volumes/)).toBeTruthy();
  });

  it("filters the blenders by name and says how many of them are left", () => {
    const { getByLabelText, getByText, queryByText } = renderView(
      mockArchiveShadersDescription({
        blenders: [
          mockArchiveShadersBlender(),
          mockArchiveShadersBlender({ name: "lights\\lights_hemi", properties: [] }),
        ],
      })
    );

    expect(getByText("Blenders (2)")).toBeTruthy();

    fireEvent.change(getByLabelText("Filter blenders"), { target: { value: "lights" } });

    expect(getByText("Blenders (1 of 2)")).toBeTruthy();
    expect(queryByText("models\\model_aref")).toBeNull();
  });
});
