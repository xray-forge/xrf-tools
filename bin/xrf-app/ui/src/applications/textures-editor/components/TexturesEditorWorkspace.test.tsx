import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Binding, Container } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { EAssetsBulkRoute } from "@xrf/ipc/commands/assets-bulk";
import { ETexturesCommand } from "@xrf/ipc/commands/textures";
import { IpcCommandMocks, mockIpcBulkRoutes, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import {
  TextureDescription,
  TextureDescriptorForm,
  TextureEncodingComparison,
  TextureMakeBumpOutcome,
  TextureSaveOutcome,
  TexturesSaveRequest,
} from "@xrf/ipc/types/xrf-app-textures";
import { Nullable } from "@xrf/types";

import { TEXTURES_EDITOR_APPLICATION } from "@/applications/textures-editor/application";
import { ITextureBumpSources, TextureBumpService } from "@/applications/textures-editor/services/texture-bump";
import { TextureEditorService } from "@/applications/textures-editor/services/texture-editor";
import { TextureEncodingService } from "@/applications/textures-editor/services/texture-encoding";
import { EditorLifecycleService } from "@/core/shell/services/editor-lifecycle";
import {
  MOCK_TEXTURE,
  mockTextureDescription,
  mockTextureEncodingComparison,
  mockTextureVocabulary,
} from "@/core/textures/fixtures/texture.mocks";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { mockContainer } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TexturesEditorWorkspace } from "./TexturesEditorWorkspace";

const SOURCES: ITextureBumpSources = {
  gloss: { kind: "constant", level: 0.5 },
  height: "C:\\work\\wall_height.png",
  normalMap: null,
  virtualHeight: 0.05,
};

function describedTexture(overrides: Partial<TextureDescription> = {}): TextureDescription {
  return mockTextureDescription(MOCK_TEXTURE, {
    targets: {
      descriptor: { expected: null, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.thm` },
      texture: { expected: { modifiedMs: 1, size: 3 }, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds` },
    },
    ...overrides,
  });
}

async function renderWorkspace(
  description: TextureDescription = describedTexture(),
  responses: IpcCommandMocks = {}
): Promise<{
  render: RenderResult;
  container: Container;
  editorService: TextureEditorService;
  selectionService: TextureSelectionService;
}> {
  // A save is answered by re-describing what it wrote, which is how the editor learns it is clean again. A fixture
  // that kept answering with the pre-save descriptor would leave every save looking refused.
  let written: Nullable<TextureDescriptorForm> = null;

  mockIpcCommands({
    [ETexturesCommand.DESCRIBE]: (): TextureDescription =>
      written ? { ...description, form: written, hasDescriptor: true } : description,
    [ETexturesCommand.GET_VOCABULARY]: mockTextureVocabulary(),
    [ETexturesCommand.SAVE]: (args?: Record<string, unknown>): TextureSaveOutcome => {
      written = (args?.request as TexturesSaveRequest).descriptor.form;

      return { outcome: "completed", written: [] };
    },
    ...responses,
  });

  mockIpcBulkRoutes({
    [EAssetsBulkRoute.READ_TEXTURE]: new ArrayBuffer(0),
  });

  // Built from the application's own bindings, so a workspace that grows a dependency cannot pass here while
  // throwing on mount in the running app.
  const runtime = await TEXTURES_EDITOR_APPLICATION.load?.();
  const container: Container = mockContainer([...((runtime?.container?.bindings ?? []) as Array<Binding>)]);
  const selectionService: TextureSelectionService = container.get(TextureSelectionService);

  await selectionService.openFile(`C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds`);

  // On the editor's own route: the toolbar names its back control after the application it resolves from the path,
  // so off-route there is no way back to click.
  const render: RenderResult = renderWithProviders(<TexturesEditorWorkspace />, {
    container,
    route: "/textures-editor",
  });

  await act(async () => {
    await Promise.resolve();
  });

  return { container, editorService: container.get(TextureEditorService), render, selectionService };
}

describe("TexturesEditorWorkspace", () => {
  beforeEach(() => resetMockInvoke());

  it("binds the draft itself, so it does not depend on which panel is on screen", async () => {
    // The draft used to be made by the descriptor panel's effect, which meant somebody who opened the editor on any
    // other panel had none - and a save, and the prompt protecting it, had nothing to act on.
    const { editorService } = await renderWorkspace();

    expect(editorService.draft).not.toBeNull();
  });

  it("closes the texture without a word when nothing is pending", async () => {
    const { render, selectionService } = await renderWorkspace();

    await userEvent.click(render.getByRole("button", { name: "Back to Textures editor" }));

    expect(selectionService.selected.value).toBeNull();
  });

  it("asks before closing a texture with edits in hand", async () => {
    const { editorService, render, selectionService } = await renderWorkspace();

    act(() => editorService.edit({ bumpName: "ston\\other_bump" }));

    await userEvent.click(render.getByRole("button", { name: "Back to Textures editor" }));

    expect(render.getByText("Leave without saving?")).toBeInTheDocument();
    expect(selectionService.selected.value).not.toBeNull();

    await userEvent.click(render.getByText("Discard and leave"));

    expect(selectionService.selected.value).toBeNull();
  });

  it("offers to write the draft on the way out", async () => {
    const { editorService, render, selectionService } = await renderWorkspace();

    act(() => editorService.edit({ bumpName: "ston\\other_bump" }));

    await userEvent.click(render.getByRole("button", { name: "Back to Textures editor" }));
    await userEvent.click(render.getByText("Save and leave"));

    await waitFor(() => expect(selectionService.selected.value).toBeNull());
  });

  it("offers only discarding for a texture served out of an archive", async () => {
    // There is no file to replace, so a save button would be a promise the editor cannot keep.
    const { editorService, render } = await renderWorkspace(describedTexture({ targets: null }));

    act(() => editorService.edit({ bumpName: "ston\\other_bump" }));

    await userEvent.click(render.getByRole("button", { name: "Back to Textures editor" }));

    expect(render.getByText("Leave without saving?")).toBeInTheDocument();
    expect(render.queryByText("Save and leave")).not.toBeInTheDocument();
  });

  it("stays open while a bump pair is being generated", async () => {
    // The pair is written beside the texture when the job finishes, and the descriptor is pointed at it only while the
    // editor is still there to take the edit.
    const answer = deferred<TextureMakeBumpOutcome>();
    const {
      container,
      render: { getByRole },
      selectionService,
    } = await renderWorkspace(describedTexture(), { [ETexturesCommand.MAKE_BUMP]: () => answer.promise });
    const lifecycle: EditorLifecycleService = container.get(EditorLifecycleService);

    let generation: Promise<void> = Promise.resolve();

    act(() => {
      generation = flowResult(container.get(TextureBumpService).run(SOURCES));
    });

    await waitFor(() => expect(lifecycle.isBusy).toBe(true));

    await userEvent.click(getByRole("button", { name: "Back to Textures editor" }));

    expect(selectionService.selected.value).not.toBeNull();

    await act(async () => {
      answer.resolve({
        bump: `C:\\gamedata\\textures\\${MOCK_TEXTURE}_bump.dds`,
        companion: `C:\\gamedata\\textures\\${MOCK_TEXTURE}_bump#.dds`,
        glossPower: 0.5,
        isGlossTooDark: false,
        outcome: "completed",
      });

      await generation;
    });

    expect(lifecycle.isBusy).toBe(false);
  });

  it("stays open while encodings are being compared", async () => {
    const answer = deferred<TextureEncodingComparison>();
    const {
      container,
      render: { getByRole },
      selectionService,
    } = await renderWorkspace(describedTexture(), { [ETexturesCommand.COMPARE_ENCODINGS]: () => answer.promise });
    const lifecycle: EditorLifecycleService = container.get(EditorLifecycleService);

    let comparison: Promise<void> = Promise.resolve();

    act(() => {
      comparison = flowResult(container.get(TextureEncodingService).run(null));
    });

    await waitFor(() => expect(lifecycle.isBusy).toBe(true));

    await userEvent.click(getByRole("button", { name: "Back to Textures editor" }));

    expect(selectionService.selected.value).not.toBeNull();

    await act(async () => {
      answer.resolve(mockTextureEncodingComparison());

      await comparison;
    });

    expect(lifecycle.isBusy).toBe(false);
  });
});
