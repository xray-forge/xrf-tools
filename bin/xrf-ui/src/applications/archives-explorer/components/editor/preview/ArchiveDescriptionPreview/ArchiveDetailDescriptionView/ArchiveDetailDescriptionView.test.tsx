import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { ArchiveDescribeScope, ArchiveDetailModel, EArchiveDescribeScope } from "@/core/ipc/types/xrf-app";
import { mockArchiveDetailModel } from "@/fixtures/mocks/archive.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveDetailDescriptionView } from "./ArchiveDetailDescriptionView";

const SCOPE: ArchiveDescribeScope = { kind: EArchiveDescribeScope.VOLUMES, volumes: 3 };

function renderView(description: ArchiveDetailModel = mockArchiveDetailModel()): RenderResult {
  const { container } = mockInjectedService(ArchivesService);

  return renderWithProviders(<ArchiveDetailDescriptionView description={description} scope={SCOPE} />, { container });
}

describe("ArchiveDetailDescriptionView", () => {
  it("offers the texture it draws with as somewhere to go", () => {
    const { getByText } = renderView();

    expect(getByText("detail\\detail_grass").closest("button")).not.toBeNull();
  });

  it("gives the shader as plain text, because it names no file to select", () => {
    const { getByText } = renderView();

    expect(getByText("details\\blend").closest("button")).toBeNull();
    expect(getByText(/rather than a file, so there is nothing to select/)).toBeTruthy();
  });

  it("states the mesh with the box it occupies as authored", () => {
    const { getByText } = renderView();

    expect(getByText("12 triangles over 24 vertices")).toBeTruthy();
    expect(getByText("0.35 × 0.60 × 0.35 m")).toBeTruthy();
  });

  it("says an object carrying no mesh has none rather than showing an empty box", () => {
    const { getByText } = renderView(mockArchiveDetailModel(null, { vertices: 0, triangles: 0, bounds: null }));

    expect(getByText("0 triangles over 0 vertices")).toBeTruthy();
    expect(getByText("The object carries no mesh at all")).toBeTruthy();
  });

  it("reads a scale of one value as that value rather than as a range of it with itself", () => {
    const { getByText } = renderView(mockArchiveDetailModel(null, { minScale: 1, maxScale: 1 }));

    expect(getByText("1.00")).toBeTruthy();
  });

  it("explains swaying from the flag that turns it off", () => {
    const { getByText } = renderView();

    expect(getByText("Yes")).toBeTruthy();
    expect(getByText(/moves it with the wind/)).toBeTruthy();
  });

  it("names the flag by the engine's own spelling where it is set", () => {
    const { getByText } = renderView(mockArchiveDetailModel(null, { isWaving: false }));

    expect(getByText("No")).toBeTruthy();
    expect(getByText(/declares `DO_NO_WAVING`/)).toBeTruthy();
  });

  it("carries a flag bit nothing claims rather than dropping it", () => {
    const { getByText } = renderView(mockArchiveDetailModel(null, { unnamedFlags: 0x10 }));

    expect(getByText("0x10")).toBeTruthy();
  });

  it("says plainly when an object names no texture", () => {
    const { getByText } = renderView(mockArchiveDetailModel(null));

    expect(getByText("Not declared")).toBeTruthy();
  });
});
