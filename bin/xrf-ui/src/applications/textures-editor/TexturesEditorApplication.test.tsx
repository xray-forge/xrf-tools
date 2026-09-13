import { describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { Binding, Container } from "@wirestate/core";

import { TEXTURES_EDITOR_APPLICATION } from "@/applications/textures-editor/application";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { TextureVocabulary } from "@/core/ipc/types/xrf-app";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription, mockTextureVocabulary } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TexturesEditorApplication } from "./TexturesEditorApplication";

const VOCABULARY: TextureVocabulary = mockTextureVocabulary();

/**
 * Builds the container out of the application's deferred runtime bindings.
 *
 * @returns A test container with the application's services.
 */
async function mockApplicationContainer(): Promise<Container> {
  const runtime = await TEXTURES_EDITOR_APPLICATION.load?.();

  return mockContainer([...((runtime?.container?.bindings ?? []) as Array<Binding>)]);
}

describe("TexturesEditorApplication", () => {
  it("binds no catalog service, because it never lists a root", async () => {
    // The structural half of "this is not the explorer with a save button": a tool that cannot resolve a catalog
    // service cannot grow a tree by accident.
    const runtime = await TEXTURES_EDITOR_APPLICATION.load?.();
    const bindings: Array<Binding> = (runtime?.container?.bindings ?? []) as Array<Binding>;

    expect(bindings).toContain(TextureSelectionService);
    expect(bindings).not.toContain(TextureCatalogService);
  });

  it("opens on the picker, with no session to restore", async () => {
    // No readiness gate either: the backend parks a browsed root set, this tool never browses one, so the picker is
    // the correct first screen every time rather than something shown after asking.
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:textures|get_vocabulary"]: VOCABULARY });

    const container: Container = await mockApplicationContainer();
    const render: RenderResult = renderWithProviders(<TexturesEditorApplication />, { container });

    expect(await render.findByText("Open a texture to work on")).toBeInTheDocument();
  });

  it("mounts the workbench over one opened file, with every service its components inject", async () => {
    resetMockInvoke();
    setMockInvokeResponses({
      ["plugin:textures|describe"]: mockTextureDescription(),
      ["plugin:textures|get_vocabulary"]: VOCABULARY,
      ["plugin:textures|read_texture"]: new ArrayBuffer(0),
    });

    const container: Container = await mockApplicationContainer();
    const selectionService: TextureSelectionService = container.get(TextureSelectionService);

    await container.get(TextureEditorService).onProvision();

    // Inside `act`, because opening settles two flows - the descriptor and the decoded preview - and each one
    // re-renders through mobx after the await the test is holding.
    await act(async () => {
      await selectionService.openFile("C:\\gamedata\\textures\\ston\\ston_beton05.dds");
    });

    const render: RenderResult = renderWithProviders(<TexturesEditorApplication />, { container });

    // A missing binding throws on mount here rather than failing an assertion further down.
    await waitFor(() => expect(render.getByTestId("textures-editor-application")).toBeInTheDocument());
    await waitFor(() => expect(selectionService.preview.isLoading).toBe(false));

    expect(selectionService.reference).toBe(MOCK_TEXTURE);
  });
});
