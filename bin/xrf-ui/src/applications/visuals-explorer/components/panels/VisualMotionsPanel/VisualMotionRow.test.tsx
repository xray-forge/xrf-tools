import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { VisualMotionRow } from "@/applications/visuals-explorer/components/panels/VisualMotionsPanel/VisualMotionRow";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { renderWithProviders } from "@/fixtures/utils/render";

function mockMotion(logicalPath: string): XrayAsset {
  return { container: { kind: "archive", path: "C:\\game\\db" }, logicalPath };
}

describe("VisualMotionRow", () => {
  it("names every file a masked reference resolved to, not just how many", () => {
    // A mask answering with the wrong set looks identical to the right one from a count alone.
    const render: RenderResult = renderWithProviders(
      <VisualMotionRow
        motion={{
          reference: "wpn\\wpn_ak74_*",
          resolution: {
            kind: "resolved",
            step: "installation",
            assets: [mockMotion("meshes\\wpn_ak74_idle.omf"), mockMotion("meshes\\wpn_ak74_reload.omf")],
          },
        }}
      />
    );

    expect(render.getByText("2 files")).toBeInTheDocument();
    expect(render.getByText("meshes\\wpn_ak74_idle.omf")).toBeInTheDocument();
    expect(render.getByText("meshes\\wpn_ak74_reload.omf")).toBeInTheDocument();
  });

  it("has no paths to name when the reference found nothing", () => {
    const render: RenderResult = renderWithProviders(
      <VisualMotionRow
        motion={{ reference: "wpn\\wpn_ak74_*", resolution: { kind: "missing", roots: ["C:\\game"] } }}
      />
    );

    expect(render.getByText("Not found")).toBeInTheDocument();
    expect(render.queryByText(/\.omf$/)).not.toBeInTheDocument();
  });
});
