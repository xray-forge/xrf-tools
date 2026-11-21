import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { exists } from "@tauri-apps/plugin-fs";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Binding, Container } from "@wirestate/core";
import { EAssetsBulkRoute } from "@xrf/ipc/commands/assets-bulk";
import { ETexturesCommand } from "@xrf/ipc/commands/textures";
import { mockInvoke, mockIpcBulkRoutes, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import {
  ETextureReferenceField,
  ETextureSaveAsBump,
  ETextureSaveAsRole,
  TextureSaveAsOutcome,
  TextureSaveAsPlan,
} from "@xrf/ipc/types/xrf-app-textures";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";

import { clearLocalStorage } from "@/core/storage";
import { MOCK_TEXTURE, mockTextureDescription, mockTextureVocabulary } from "@/core/textures/fixtures/texture.mocks";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { TextureEditorService } from "@/tools/textures-editor/services/texture-editor";
import { TEXTURES_EDITOR_TOOL } from "@/tools/textures-editor/tool";

import { TextureSaveAsDialog } from "./TextureSaveAsDialog";

const COPY: string = `${MOCK_TEXTURE}_copy`;

// One field of the two that name textures points at the original, so exactly one row is listed.
const PLAN: TextureSaveAsPlan = {
  destination: COPY,
  hasBumpPair: true,
  files: [
    {
      role: ETextureSaveAsRole.TEXTURE,
      from: `textures\\${MOCK_TEXTURE}.dds`,
      to: `textures\\${COPY}.dds`,
      isReplacing: false,
    },
    { role: ETextureSaveAsRole.DESCRIPTOR, from: null, to: `textures\\${COPY}.thm`, isReplacing: true },
  ],
  renames: [{ field: ETextureReferenceField.BUMP, from: `${MOCK_TEXTURE}_bump`, to: `${COPY}_bump` }],
};

const OUTCOME: TextureSaveAsOutcome = {
  outcome: EJobOutcome.COMPLETED,
  reference: COPY,
  written: [`C:\\gamedata\\textures\\${COPY}.dds`, `C:\\gamedata\\textures\\${COPY}.thm`],
};

function plans(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === ETexturesCommand.PLAN_SAVE_AS).length;
}

async function renderDialog(): Promise<RenderResult> {
  mockIpcCommands({
    [ETexturesCommand.DESCRIBE]: mockTextureDescription(MOCK_TEXTURE, {
      targets: {
        descriptor: { expected: null, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.thm` },
        texture: { expected: { modifiedMs: 1, size: 3 }, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds` },
      },
    }),
    [ETexturesCommand.GET_VOCABULARY]: mockTextureVocabulary(),
    [ETexturesCommand.PLAN_SAVE_AS]: PLAN,
    [ETexturesCommand.SAVE_AS]: OUTCOME,
  });

  mockIpcBulkRoutes({
    [EAssetsBulkRoute.READ_TEXTURE]: new ArrayBuffer(0),
  });

  // From the tool's own bindings, as the dialog resolves its services through what the tool provides.
  const runtime = await TEXTURES_EDITOR_TOOL.load?.();
  const container: Container = mockContainer([...((runtime?.container?.bindings ?? []) as Array<Binding>)]);

  const selectionService: TextureSelectionService = container.get(TextureSelectionService);

  await selectionService.openFile(`C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds`);

  // As the workspace does: a copy is of the draft, which the dialog reads rather than makes.
  container.get(TextureEditorService).bind(selectionService.selected.value);

  const render: RenderResult = renderWithProviders(<TextureSaveAsDialog isOpen onClose={() => {}} />, {
    container,
    route: "/textures-editor",
  });

  await act(async () => {
    await Promise.resolve();
  });

  return render;
}

describe("TextureSaveAsDialog", () => {
  beforeEach(() => {
    resetMockInvoke();
    clearLocalStorage();
    jest.mocked(exists).mockResolvedValue(true);
  });

  it("lists every file it writes, and under Advanced exactly the fields it changes", async () => {
    const { findByTestId, getAllByTestId, getByRole, getByTestId } = await renderDialog();

    await findByTestId("texture-save-as-advanced");
    await userEvent.click(getByRole("button", { name: "Advanced" }));

    expect(getAllByTestId("texture-save-as-rename-field").map((row: HTMLElement) => row.textContent)).toEqual([
      `Bump texture: ${MOCK_TEXTURE}_bump → ${COPY}_bump`,
    ]);
    expect(getByTestId("texture-save-as-files")).toHaveTextContent(`Texture: textures\\${COPY}.dds`);
    expect(getByTestId("texture-save-as-files")).toHaveTextContent(
      `Descriptor: textures\\${COPY}.thm (replaces an existing file)`
    );

    // The plan is asked for the name the dialog starts from: the original's, marked as a copy.
    expect(mockInvoke).toHaveBeenCalledWith(
      ETexturesCommand.PLAN_SAVE_AS,
      expect.objectContaining({
        request: expect.objectContaining({ target: expect.objectContaining({ destination: COPY }) }),
      })
    );
  });

  it("keeps what it shows while a name is typed, and plans the name once typing pauses", async () => {
    const { findByTestId, getByRole, getByTestId } = await renderDialog();

    await findByTestId("texture-save-as-files");

    const planned: number = plans();

    await userEvent.type(getByRole("textbox", { name: "Name" }), "_bc7");

    // An emptied list would collapse the dialog and grow it back on every key.
    expect(getByTestId("texture-save-as-files")).toBeInTheDocument();
    expect(getByTestId("texture-save-as-confirm")).toBeDisabled();

    await waitFor(() => expect(plans()).toBe(planned + 1));
    await waitFor(() => expect(getByTestId("texture-save-as-confirm")).toBeEnabled());
  });

  it("copies the bump map unless told to share it, and checks and sends that choice", async () => {
    const { findByTestId, getByRole, getByTestId } = await renderDialog();

    await findByTestId("texture-save-as-advanced");
    await userEvent.click(getByRole("button", { name: "Advanced" }));

    expect(getByRole("button", { name: "Copy it" })).toHaveAttribute("aria-pressed", "true");

    await userEvent.click(getByRole("button", { name: "Share the original's" }));

    // A changed choice changes the files written, so it is checked before Save as is offered again.
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith(
        ETexturesCommand.PLAN_SAVE_AS,
        expect.objectContaining({ request: expect.objectContaining({ bump: ETextureSaveAsBump.SHARED }) })
      )
    );
    await waitFor(() => expect(getByTestId("texture-save-as-confirm")).toBeEnabled());
    await userEvent.click(getByTestId("texture-save-as-confirm"));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith(
        ETexturesCommand.SAVE_AS,
        expect.objectContaining({
          request: expect.objectContaining({ bump: ETextureSaveAsBump.SHARED, isForced: false }),
        })
      )
    );
  });
});
