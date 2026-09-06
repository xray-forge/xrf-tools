import { describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { Binding, Container } from "@wirestate/core";

import { TEXTURES_EXPLORER_APPLICATION } from "@/applications/textures-explorer/application";
import { TexturesExplorerApplication } from "@/applications/textures-explorer/TexturesExplorerApplication";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  MOCK_TEXTURE,
  mockBumpedTextureSummary,
  mockTextureCatalog,
  mockTextureDescription,
  mockTextureEntry,
} from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

/**
 * Builds the container out of the application's deferred runtime bindings.
 *
 * @returns A test container with the application's services.
 */
async function mockApplicationContainer(): Promise<Container> {
  const runtime = await TEXTURES_EXPLORER_APPLICATION.load?.();

  return mockContainer([...((runtime?.container?.bindings ?? []) as Array<Binding>)]);
}

describe("TexturesExplorerApplication", () => {
  it("renders the picker once the backend has been asked", async () => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:textures|get_roots"]: null });

    const container: Container = await mockApplicationContainer();

    await container.get(TextureCatalogService).onProvision();

    const render: RenderResult = renderWithProviders(<TexturesExplorerApplication />, { container });

    expect(await render.findByText("Open game textures")).toBeInTheDocument();
  });

  it("renders the editor over a browsed root, with every service its components inject", async () => {
    resetMockInvoke();
    setMockInvokeResponses({
      ["plugin:textures|describe"]: mockTextureDescription(),
      ["plugin:textures|describe_catalog"]: [mockBumpedTextureSummary()],
      ["plugin:textures|get_roots"]: null,
      ["plugin:textures|open"]: mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)]),
      ["plugin:textures|read_texture"]: new ArrayBuffer(0),
    });

    const container: Container = await mockApplicationContainer();
    const service: TextureCatalogService = container.get(TextureCatalogService);
    const selectionService: TextureSelectionService = container.get(TextureSelectionService);

    await service.onProvision();
    await service.openRoot("C:\\gamedata");

    const render: RenderResult = renderWithProviders(<TexturesExplorerApplication />, { container });

    // The workspace mounts its viewport and its panels; a missing binding throws here rather than failing an
    // assertion.
    await waitFor(() => expect(render.getByTestId("textures-explorer-application")).toBeInTheDocument());

    // Inside `act`, because selecting settles two flows - the descriptor and the decoded preview - and each one
    // re-renders the workspace through mobx after the await the test is holding.
    await act(async () => {
      await service.select(MOCK_TEXTURE);
    });

    await waitFor(() => expect(selectionService.preview.isLoading).toBe(false));

    expect(selectionService.reference).toBe(MOCK_TEXTURE);
  });
});
