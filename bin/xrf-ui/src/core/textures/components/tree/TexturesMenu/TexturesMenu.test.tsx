import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { TextureCatalog, TextureMaterialSummary } from "@/core/ipc/types/xrf-app";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  MOCK_BUMP,
  MOCK_COMPANION,
  MOCK_TEXTURE,
  mockBumpedTextureSummary,
  mockTextureBadges,
  mockTextureCatalog,
  mockTextureDescription,
  mockTextureEntry,
  mockTextureSummary,
} from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TexturesMenu } from "./TexturesMenu";

async function renderMenu(
  catalog: TextureCatalog,
  summaries: Array<TextureMaterialSummary>
): Promise<{ render: RenderResult; container: Container }> {
  resetMockInvoke();

  setMockInvokeResponses({
    ["plugin:textures|describe"]: mockTextureDescription(),
    ["plugin:textures|describe_catalog"]: summaries,
    ["plugin:textures|get_roots"]: null,
    ["plugin:textures|open"]: mockSessionResponse(catalog),
  });

  // Both halves: the catalog lists, and choosing a row hands the reference to the selection.
  const container: Container = mockContainer([TextureSelectionService, TextureCatalogService]);

  await container.get(TextureCatalogService).openRoot("C:\\gamedata");

  return { container, render: renderWithProviders(<TexturesMenu />, { container }) };
}

function mockPairCatalog(): TextureCatalog {
  return mockTextureCatalog([
    mockTextureEntry(MOCK_TEXTURE),
    mockTextureEntry(MOCK_BUMP),
    mockTextureEntry(MOCK_COMPANION),
  ]);
}

describe("TexturesMenu", () => {
  it("renders the listing as a tree of engine references", async () => {
    const { render } = await renderMenu(mockPairCatalog(), [mockBumpedTextureSummary()]);

    expect(render.getByRole("heading", { name: "Textures" })).toBeInTheDocument();
    expect(render.getByText("ston")).toBeInTheDocument();
  });

  it("draws no row for a bump half its texture declares", async () => {
    const { render } = await renderMenu(mockPairCatalog(), [mockBumpedTextureSummary()]);

    fireEvent.dblClick(render.getByText("ston"));

    expect(await render.findByText("ston_beton05")).toBeInTheDocument();
    expect(render.queryByText("ston_beton05_bump")).not.toBeInTheDocument();
    expect(render.queryByText("ston_beton05_bump#")).not.toBeInTheDocument();
  });

  it("draws a row for a bump half nothing declares", async () => {
    const { render } = await renderMenu(
      mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE), mockTextureEntry("ston\\ston_orphan_bump")]),
      [mockTextureSummary(MOCK_TEXTURE)]
    );

    fireEvent.dblClick(render.getByText("ston"));

    expect(await render.findByText("ston_orphan_bump")).toBeInTheDocument();
  });

  it("counts what each filter would show and narrows the tree to it", async () => {
    const { render } = await renderMenu(
      mockTextureCatalog([mockTextureEntry("ston\\healthy"), mockTextureEntry("ston\\broken")]),
      [
        mockTextureSummary("ston\\healthy", { badges: mockTextureBadges({ isBumped: true }) }),
        mockTextureSummary("ston\\broken", { badges: mockTextureBadges({ isDegraded: true }) }),
      ]
    );

    const filter: HTMLElement = await render.findByText("Degraded 1");

    fireEvent.click(filter);
    fireEvent.dblClick(render.getByText("ston"));

    expect(await render.findByText("broken")).toBeInTheDocument();
    expect(render.queryByText("healthy")).not.toBeInTheDocument();
  });

  it("says why the tree is empty when a filter matches nothing", async () => {
    const { render } = await renderMenu(mockTextureCatalog([mockTextureEntry("ston\\healthy")]), [
      mockTextureSummary("ston\\healthy", { badges: mockTextureBadges({ isBumped: true }) }),
    ]);

    fireEvent.click(await render.findByText("Bumped 1"));
    fireEvent.click(render.getByText("Bumped 1"));

    // Unselected again, so the tree is back rather than reporting an empty filter.
    expect(render.getByText("ston")).toBeInTheDocument();
  });
});
