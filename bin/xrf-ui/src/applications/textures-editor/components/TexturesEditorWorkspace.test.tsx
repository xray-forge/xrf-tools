import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Binding, Container } from "@wirestate/core";

import { TEXTURES_EDITOR_APPLICATION } from "@/applications/textures-editor/application";
import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import {
  TextureDescription,
  TextureDescriptorForm,
  TextureSaveOutcome,
  TexturesSaveRequest,
} from "@/core/bindings/types/xrf-app";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription, mockTextureVocabulary } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { Nullable } from "@/lib/types/general";

import { TexturesEditorWorkspace } from "./TexturesEditorWorkspace";

/** A described texture with somewhere to write to, which is what makes a save possible at all. */
function describedTexture(overrides: Partial<TextureDescription> = {}): TextureDescription {
  return mockTextureDescription(MOCK_TEXTURE, {
    targets: {
      descriptor: { expected: null, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.thm` },
      texture: { expected: { modifiedMs: 1, size: 3 }, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds` },
    },
    ...overrides,
  });
}

async function renderWorkspace(description: TextureDescription = describedTexture()): Promise<{
  render: RenderResult;
  editorService: TextureEditorService;
  selectionService: TextureSelectionService;
}> {
  // A save is answered by re-describing what it wrote, which is how the editor learns it is clean again. A fixture
  // that kept answering with the pre-save descriptor would leave every save looking refused.
  let written: Nullable<TextureDescriptorForm> = null;

  setMockInvokeResponses({
    ["plugin:textures|describe"]: (): TextureDescription => ({ ...description, form: written ?? description.form }),
    ["plugin:textures|get_roots"]: null,
    ["plugin:textures|get_vocabulary"]: mockTextureVocabulary(),
    ["plugin:textures|read_texture"]: new ArrayBuffer(0),
    ["plugin:textures|save"]: (args?: Record<string, unknown>): TextureSaveOutcome => {
      written = (args?.request as TexturesSaveRequest).descriptor?.form ?? null;

      return { descriptorFormat: null, outcome: "completed", written: [] };
    },
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

  return { editorService: container.get(TextureEditorService), render, selectionService };
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
});
