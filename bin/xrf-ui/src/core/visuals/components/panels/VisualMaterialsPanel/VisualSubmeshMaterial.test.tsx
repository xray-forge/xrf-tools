import { describe, expect, it } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

import { VisualSubmeshMaterial } from "./VisualSubmeshMaterial";

const THM: XrayAsset = {
  container: { kind: "directory", relativePath: "textures\\wpn\\wpn_ak74.thm", root: "C:\\gamedata" },
  logicalPath: "textures\\wpn\\wpn_ak74.thm",
};

const DUMMY_COMPANION: XrayAsset = {
  container: { kind: "directory", relativePath: "textures\\ed\\ed_dummy_bump#.dds", root: "C:\\gamedata" },
  logicalPath: "textures\\ed\\ed_dummy_bump#.dds",
};

function render(material: XrayMaterialDescriptor): RenderResult {
  return renderWithProviders(<VisualSubmeshMaterial material={material} />);
}

describe("VisualSubmeshMaterial", () => {
  it("costs one chip and no rows for a texture with no descriptor", () => {
    // The common case: most textures declare nothing, and a wall of rows saying so would bury the ones that do.
    const { getByText, queryByText } = render({
      descriptor: null,
      declaration: { kind: "noDescriptor" },
      bump: null,
      outcome: "flat",
      detail: null,
    });

    expect(getByText("Flat")).toBeInTheDocument();
    expect(queryByText("Declared by")).not.toBeInTheDocument();
    expect(queryByText("Bump map")).not.toBeInTheDocument();
    expect(queryByText("Height")).not.toBeInTheDocument();
  });

  it("names both inputs and where they resolved for a bumped material", () => {
    const { getByText } = render(mockMaterialDescriptor());

    expect(getByText("Bumped")).toBeInTheDocument();
    expect(getByText("textures\\wpn\\wpn_ak74.thm, mode 'Use'")).toBeInTheDocument();
    expect(getByText("wpn\\wpn_ak74_bump · Resolved in asset root")).toBeInTheDocument();
    expect(getByText("wpn\\wpn_ak74_bump# · Resolved in asset root")).toBeInTheDocument();
  });

  it("says the height is authoring data the renderer never reads", () => {
    // The trap the row exists for: the SDK exposes Virtual Height, and changing it changes nothing in the game.
    const { getByText } = render(mockMaterialDescriptor());

    expect(getByText("0.050 m · authoring only, not read by the renderer")).toBeInTheDocument();
  });

  it("warns on a dummy and names the file the engine actually binds", () => {
    const material: XrayMaterialDescriptor = mockMaterialDescriptor({ outcome: "dummy" });

    material.bump!.companion.resolution = {
      kind: "substituted",
      step: "asset root",
      fallback: "ed\\ed_dummy_bump#",
      assets: [DUMMY_COMPANION],
    };

    const { getByText } = render(material);

    expect(getByText("Dummy bump")).toBeInTheDocument();
    expect(
      getByText(
        "wpn\\wpn_ak74_bump# · Missing, showing the engine placeholder from asset root " +
          "(textures\\ed\\ed_dummy_bump#.dds)"
      )
    ).toBeInTheDocument();
  });

  it("reports a missing pair as an error", () => {
    const material: XrayMaterialDescriptor = mockMaterialDescriptor({ outcome: "missing" });

    material.bump!.bump.resolution = { kind: "missing", roots: ["C:\\gamedata"] };

    const { getByText } = render(material);

    expect(getByText("Bump missing")).toBeInTheDocument();
    expect(getByText("wpn\\wpn_ak74_bump · Not present in any searched source")).toBeInTheDocument();
  });

  it("tells a type-disqualified declaration apart from a flat one", () => {
    // Everything in the bump chunk is right; the engine skips the descriptor for its type. A hex editor cannot show it.
    const { getByText } = render({
      descriptor: THM,
      declaration: {
        kind: "typeDisqualified",
        textureType: 2,
        label: "Bump Map",
        declaredBump: "wpn\\wpn_ak74_bump",
      },
      bump: null,
      outcome: "flat",
      detail: null,
    });

    expect(getByText("Flat")).toBeInTheDocument();
    expect(
      getByText(
        "textures\\wpn\\wpn_ak74.thm declares 'wpn\\wpn_ak74_bump', but its type 'Bump Map' is skipped by the engine"
      )
    ).toBeInTheDocument();
  });

  it("names each of the other flat declarations", () => {
    const cases: Array<[XrayMaterialDescriptor["declaration"], string]> = [
      [{ kind: "noBumpChunk" }, "textures\\wpn\\wpn_ak74.thm carries no bump chunk"],
      [{ kind: "disabled", mode: 1 }, "textures\\wpn\\wpn_ak74.thm sets bump mode to none"],
      [
        { kind: "emptyName", mode: "parallax" },
        "textures\\wpn\\wpn_ak74.thm asks for 'Use parallax' with an empty bump name",
      ],
      [{ kind: "unreadable", reason: "not a chunk" }, "textures\\wpn\\wpn_ak74.thm could not be read: not a chunk"],
    ];

    for (const [declaration, expected] of cases) {
      const { getByText, unmount } = render({
        descriptor: THM,
        declaration,
        bump: null,
        outcome: "flat",
        detail: null,
      });

      expect(getByText(expected)).toBeInTheDocument();
      unmount();
    }
  });

  it("shows a detail association, and says when the engine does not apply it", () => {
    const applied = render(
      mockMaterialDescriptor({ detail: { name: "detail\\detail_grnd_grass", scale: 4, usage: "bump" } })
    );

    expect(applied.getByText("detail\\detail_grnd_grass · ×4.0 · bump")).toBeInTheDocument();
    applied.unmount();

    const { getByText } = render(
      mockMaterialDescriptor({ detail: { name: "detail\\detail_grnd_grass", scale: 4, usage: null } })
    );

    expect(getByText("detail\\detail_grnd_grass · ×4.0 · not applied, no usage flag is set")).toBeInTheDocument();
  });
});
