import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { VisualSubmeshTexture } from "@/applications/visuals-explorer/components/panels/VisualMaterialsPanel/VisualSubmeshTexture";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { mockTextureDescriptor } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const LOOSE: XrayAsset = {
  container: { kind: "directory", relativePath: "textures\\wpn\\wpn_ak74.dds", root: "C:\\gamedata" },
  logicalPath: "textures\\wpn\\wpn_ak74.dds",
};

const ARCHIVED: XrayAsset = {
  container: { kind: "archive", path: "C:\\game\\db" },
  logicalPath: "textures\\wpn\\wpn_ak74.dds",
};

const SHAPE_WITHOUT_MIPS = { width: 1024, height: 1024, mipmapLevels: 1, format: "BC7_UNorm" };

const PLACEHOLDER: XrayAsset = {
  container: { kind: "directory", relativePath: "textures\\ed\\ed_not_existing_texture.dds", root: "C:\\gamedata" },
  logicalPath: "textures\\ed\\ed_not_existing_texture.dds",
};

function render(texture: VisualTextureDependency): RenderResult {
  return renderWithProviders(
    <VisualSubmeshTexture
      texture={texture}
      status={{ submeshIndex: texture.submeshIndex, state: EVisualTextureState.APPLIED, reason: null }}
    />
  );
}

describe("VisualSubmeshTexture", () => {
  it("names the file a reference resolved to, and the root it came from", () => {
    const render_: RenderResult = render({
      submeshIndex: 0,
      reference: "wpn\\wpn_ak74",
      resolution: { kind: "resolved", step: "asset", assets: [LOOSE] },
    });

    expect(render_.getByText("textures\\wpn\\wpn_ak74.dds")).toBeInTheDocument();
    expect(render_.getByText("C:\\gamedata")).toBeInTheDocument();
  });

  it("names the archived entry as well as the volume set it sits in", () => {
    // The volume alone was all the panel used to show, which said where to look but never what was found there.
    const render_: RenderResult = render({
      submeshIndex: 0,
      reference: "wpn\\wpn_ak74",
      resolution: { kind: "resolved", step: "installation", assets: [ARCHIVED] },
    });

    expect(render_.getByText("textures\\wpn\\wpn_ak74.dds")).toBeInTheDocument();
    expect(render_.getByText("C:\\game\\db")).toBeInTheDocument();
  });

  it("shows the placeholder path when the reference was substituted", () => {
    // The point of the row: what is on screen is not what the model asked for, and the path is what says so.
    const render_: RenderResult = render({
      submeshIndex: 0,
      reference: "wpn\\wpn_missing",
      resolution: {
        kind: "substituted",
        step: "asset",
        fallback: "ed\\ed_not_existing_texture",
        assets: [PLACEHOLDER],
      },
    });

    expect(render_.getByText("textures\\ed\\ed_not_existing_texture.dds")).toBeInTheDocument();
  });

  it("states what the located file is, and says no mips when there is no chain", () => {
    // A single level means no chain at all, which is why the loader has to drop to a linear filter - saying "1 mip"
    // would read as if it had one.
    const render_: RenderResult = renderWithProviders(
      <VisualSubmeshTexture
        texture={{
          submeshIndex: 0,
          reference: "wpn\\wpn_ak74",
          resolution: { kind: "resolved", step: "asset", assets: [LOOSE] },
        }}
        status={{ submeshIndex: 0, state: EVisualTextureState.APPLIED, reason: null }}
        textures={{ [LOOSE.logicalPath]: mockTextureDescriptor({ size: 2048, shape: SHAPE_WITHOUT_MIPS }) }}
      />
    );

    expect(render_.getByText("2 KB")).toBeInTheDocument();
    expect(render_.getByText("1024×1024 · BC7_UNorm · no mips")).toBeInTheDocument();
  });

  it("shows no size for a located file the backend could not describe", () => {
    // Unreachable is not empty, so the row is absent rather than reporting zero bytes.
    const render_: RenderResult = render({
      submeshIndex: 0,
      reference: "wpn\\wpn_ak74",
      resolution: { kind: "resolved", step: "asset", assets: [LOOSE] },
    });

    expect(render_.queryByText("Size")).not.toBeInTheDocument();
    expect(render_.queryByText("0 B")).not.toBeInTheDocument();
  });

  it("lists the roots it searched when nothing answered", () => {
    const render_: RenderResult = render({
      submeshIndex: 0,
      reference: "wpn\\wpn_ak74",
      resolution: { kind: "missing", roots: ["C:\\gamedata"] },
    });

    expect(render_.getByText("C:\\gamedata")).toBeInTheDocument();
    expect(render_.queryByText("Path")).not.toBeInTheDocument();
  });
});
