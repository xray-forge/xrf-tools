import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveShaderCompilerDescription } from "@/core/ipc/types/xrf-app";
import { mockArchiveShaderCompilerDescription } from "@/fixtures/mocks/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { ArchiveShaderCompilerView } from "./ArchiveShaderCompilerView";

function renderView(
  description: ArchiveShaderCompilerDescription = mockArchiveShaderCompilerDescription()
): RenderResult {
  return renderWithProviders(<ArchiveShaderCompilerView description={description} />);
}

describe("ArchiveShaderCompilerView", () => {
  it("says outright that this is the build half rather than the drawing half", () => {
    // The renderer's blender of the same name lives in shaders.xr; confusing the two is the obvious mistake here.
    const { getByText } = renderView();

    expect(getByText("Compiler shaders (2)")).toBeTruthy();
    expect(getByText(/blender of the same name in shaders.xr is what draws it/)).toBeTruthy();
  });

  it("names what the compiler was told to do with a surface", () => {
    const { getByText } = renderView();

    expect(getByText("collision, rendering, optimize UV, casts shadow, sharp light")).toBeTruthy();
  });

  it("says plainly when a shader neither collides nor is drawn, rather than showing an empty line", () => {
    const { getByText } = renderView();

    expect(getByText("Neither collides nor is drawn")).toBeTruthy();
  });

  it("gives what a surface costs to bake", () => {
    const { getByText } = renderView();

    expect(getByText("density 1.00 · translucency 0.50 · ambient 0.00")).toBeTruthy();
  });
});
