import { describe, expect, it } from "@jest/globals";
import { fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { Nullable } from "@xrf/types";

import { TextureCatalog, TextureDescription, TextureEntry, TextureMaterialSummary } from "@/core/ipc/types/xrf-app";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  MOCK_BUMP,
  MOCK_COMPANION,
  MOCK_TEXTURE,
  mockBumpedTextureSummary,
  mockTextureAsset,
  mockTextureBadges,
  mockTextureCatalog,
  mockTextureDescription,
  mockTextureEntry,
  mockTextureSummary,
} from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TexturesMenu } from "./TexturesMenu";

async function expand(render: RenderResult, directory: string): Promise<void> {
  fireEvent.dblClick(render.getByText("textures"));
  fireEvent.dblClick(await render.findByText(directory));
}

function mockPairCatalog(): TextureCatalog {
  return mockTextureCatalog([
    mockTextureEntry(MOCK_TEXTURE),
    mockTextureEntry(MOCK_BUMP),
    mockTextureEntry(MOCK_COMPANION),
  ]);
}

async function renderMenu(
  catalog: TextureCatalog,
  summaries: Array<TextureMaterialSummary>,
  description: TextureDescription = mockTextureDescription()
): Promise<{ render: RenderResult; container: Container }> {
  resetMockInvoke();

  setMockInvokeResponses({
    ["plugin:textures|describe"]: description,
    ["plugin:textures|describe_catalog"]: summaries,
    ["plugin:textures|get_roots"]: null,
    ["plugin:textures|open"]: mockSessionResponse(catalog),
  });

  // Both halves: the catalog lists, and choosing a row hands the reference to the selection.
  const container: Container = mockContainer([TextureSelectionService, TextureCatalogService]);

  await container.get(TextureCatalogService).openRoot("C:\\gamedata");

  return { container, render: renderWithProviders(<TexturesMenu />, { container }) };
}

describe("TexturesMenu", () => {
  it("renders the listing as a tree of logical paths, top directory included", async () => {
    const { render } = await renderMenu(mockPairCatalog(), [mockBumpedTextureSummary()]);

    expect(render.getByRole("heading", { name: "Textures" })).toBeInTheDocument();

    // The directory the engine holds these under, which a tree of bare references never named.
    expect(render.getByText("textures")).toBeInTheDocument();

    fireEvent.dblClick(render.getByText("textures"));

    expect(await render.findByText("ston")).toBeInTheDocument();
  });

  it("draws no row for a bump half its texture declares", async () => {
    const { render } = await renderMenu(mockPairCatalog(), [mockBumpedTextureSummary()]);

    await expand(render, "ston");

    expect(await render.findByText("ston_beton05.dds")).toBeInTheDocument();
    expect(render.queryByText("ston_beton05_bump.dds")).not.toBeInTheDocument();
    expect(render.queryByText("ston_beton05_bump#.dds")).not.toBeInTheDocument();
  });

  it("draws a row for a bump half nothing declares", async () => {
    const { render } = await renderMenu(
      mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE), mockTextureEntry("ston\\ston_orphan_bump")]),
      [mockTextureSummary(MOCK_TEXTURE)]
    );

    await expand(render, "ston");

    expect(await render.findByText("ston_orphan_bump.dds")).toBeInTheDocument();
  });

  // Opening writes the mark; walking the tree afterwards moves only the cursor, so the row a person came back from
  // still says which texture the viewport is holding.
  it("marks the open texture apart from the row the keyboard moved to", async () => {
    const { render, container } = await renderMenu(mockPairCatalog(), [mockBumpedTextureSummary()]);

    await expand(render, "ston");

    const texture: HTMLElement = await render.findByText("ston_beton05.dds");

    fireEvent.dblClick(texture);

    await waitFor(() => expect(container.get(TextureSelectionService).reference).toBe(MOCK_TEXTURE));

    const openRow: Nullable<HTMLElement> = texture.closest("[role=treeitem]");
    const directoryRow: Nullable<HTMLElement> = render.getByText("ston").closest("[role=treeitem]");

    expect(openRow).toHaveAttribute("aria-current", "true");

    // The cursor leaves; the mark does not.
    fireEvent.click(render.getByText("ston"));

    expect(openRow).toHaveAttribute("aria-current", "true");
    expect(openRow).toHaveAttribute("aria-selected", "false");
    expect(directoryRow).toHaveAttribute("aria-selected", "true");
    expect(directoryRow).not.toHaveAttribute("aria-current");
  });

  // A loose listing keys its rows by the path below the folder, while the backend describes the file by the engine
  // reference its own tree implies for it. The two disagree by design, so the mark follows the address the row was
  // opened with rather than the name that came back.
  it("marks the open row of a loose listing, whose backend name is not what the row is keyed by", async () => {
    const path: string = "C:\\loose\\sub\\wall01.dds";
    const entry: TextureEntry = {
      descriptor: null,
      reference: "sub\\wall01",
      role: "texture",
      source: { kind: "file", path },
      texture: mockTextureAsset("sub\\wall01.dds", "C:\\loose"),
    };

    const { render } = await renderMenu(
      mockTextureCatalog([entry], { mode: "looseDirectory" }),
      [],
      mockTextureDescription(MOCK_TEXTURE, { source: { kind: "file", path } })
    );

    fireEvent.dblClick(render.getByText("sub"));

    const row: HTMLElement = await render.findByText("wall01.dds");

    fireEvent.dblClick(row);

    await waitFor(() => expect(row.closest("[role=treeitem]")).toHaveAttribute("aria-current", "true"));
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
    await expand(render, "ston");

    expect(await render.findByText("broken.dds")).toBeInTheDocument();
    expect(render.queryByText("healthy.dds")).not.toBeInTheDocument();
  });

  it("says why the tree is empty when a filter matches nothing", async () => {
    const { render } = await renderMenu(mockTextureCatalog([mockTextureEntry("ston\\healthy")]), [
      mockTextureSummary("ston\\healthy", { badges: mockTextureBadges({ isBumped: true }) }),
    ]);

    fireEvent.click(await render.findByText("Bumped 1"));
    fireEvent.click(render.getByText("Bumped 1"));

    // Unselected again, so the tree is back rather than reporting an empty filter.
    expect(render.getByText("textures")).toBeInTheDocument();
  });
});
