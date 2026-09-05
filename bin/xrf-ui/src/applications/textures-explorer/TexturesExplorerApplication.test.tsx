import { describe, expect, it } from "@jest/globals";
import { RenderResult, waitFor } from "@testing-library/react";
import { Binding, Container } from "@wirestate/core";

import { TEXTURES_EXPLORER_APPLICATION } from "@/applications/textures-explorer/application";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TexturesExplorerApplication } from "@/applications/textures-explorer/TexturesExplorerApplication";
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
 * Builds the container out of the application descriptor's own bindings.
 */
function mockApplicationContainer(): Container {
  return mockContainer([...((TEXTURES_EXPLORER_APPLICATION.container?.bindings ?? []) as Array<Binding>)]);
}

describe("TexturesExplorerApplication", () => {
  it("renders the picker once the backend has been asked", async () => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:textures|get_roots"]: null });

    const container: Container = mockApplicationContainer();

    await container.get(TexturesService).onProvision();

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

    const container: Container = mockApplicationContainer();
    const service: TexturesService = container.get(TexturesService);

    await service.onProvision();
    await service.openRoot("C:\\gamedata");

    const render: RenderResult = renderWithProviders(<TexturesExplorerApplication />, { container });

    // The editor mounts its viewport and its panels; a missing binding throws here rather than failing an assertion.
    await waitFor(() => expect(render.getByTestId("textures-explorer-application")).toBeInTheDocument());

    await service.select(MOCK_TEXTURE);

    expect(service.selectedReference).toBe(MOCK_TEXTURE);
  });
});
