import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { VisualSubmeshTexture } from "@/applications/visuals-explorer/components/panels/VisualMaterialsPanel/VisualSubmeshTexture";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { renderWithProviders } from "@/fixtures/utils/render";

const LOOSE: XrayAsset = {
  container: { kind: "directory", relativePath: "textures\\wpn\\wpn_ak74.dds", root: "C:\\gamedata" },
  logicalPath: "textures\\wpn\\wpn_ak74.dds",
};

const ARCHIVED: XrayAsset = {
  container: { kind: "archive", path: "C:\\game\\db" },
  logicalPath: "textures\\wpn\\wpn_ak74.dds",
};

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
