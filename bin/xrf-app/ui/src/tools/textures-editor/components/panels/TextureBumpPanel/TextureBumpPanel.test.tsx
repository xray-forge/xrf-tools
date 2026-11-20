import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { exists } from "@tauri-apps/plugin-fs";
import { act, fireEvent, RenderResult, waitFor, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Binding, Container } from "@wirestate/core";
import { EAssetsBulkRoute } from "@xrf/ipc/commands/assets-bulk";
import { ETexturesCommand } from "@xrf/ipc/commands/textures";
import { mockInvoke, mockIpcBulkRoutes, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { TextureMakeBumpOutcome } from "@xrf/ipc/types/xrf-app-textures";
import { EGenerateBumpFormat } from "@xrf/ipc/types/xrf-texture";

import { clearLocalStorage } from "@/core/storage";
import { MOCK_TEXTURE, mockTextureDescription, mockTextureVocabulary } from "@/core/textures/fixtures/texture.mocks";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { TEXTURES_EDITOR_TOOL } from "@/tools/textures-editor/tool";

import { TextureBumpPanel } from "./TextureBumpPanel";

const OUTCOME: TextureMakeBumpOutcome = {
  bump: `C:\\gamedata\\textures\\${MOCK_TEXTURE}_bump.dds`,
  companion: `C:\\gamedata\\textures\\${MOCK_TEXTURE}_bump#.dds`,
  glossPower: 0.5,
  isGlossTooDark: false,
  outcome: "completed",
};

async function renderPanel(): Promise<RenderResult> {
  mockIpcCommands({
    [ETexturesCommand.DESCRIBE]: mockTextureDescription(MOCK_TEXTURE, {
      targets: {
        descriptor: { expected: null, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.thm` },
        texture: { expected: { modifiedMs: 1, size: 3 }, path: `C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds` },
      },
    }),
    [ETexturesCommand.GET_VOCABULARY]: mockTextureVocabulary(),
    [ETexturesCommand.MAKE_BUMP]: OUTCOME,
  });

  mockIpcBulkRoutes({
    [EAssetsBulkRoute.READ_TEXTURE]: new ArrayBuffer(0),
  });

  // From the tool's own bindings, as the panel resolves its services through what the tool provides.
  const runtime = await TEXTURES_EDITOR_TOOL.load?.();
  const container: Container = mockContainer([...((runtime?.container?.bindings ?? []) as Array<Binding>)]);

  await container.get(TextureSelectionService).openFile(`C:\\gamedata\\textures\\${MOCK_TEXTURE}.dds`);

  const render: RenderResult = renderWithProviders(<TextureBumpPanel />, { container, route: "/textures-editor" });

  await act(async () => {
    await Promise.resolve();
  });

  return render;
}

describe("TextureBumpPanel", () => {
  beforeEach(() => {
    resetMockInvoke();
    clearLocalStorage();
    jest.mocked(exists).mockResolvedValue(true);
  });

  it("remembers a typed height map once a pair is generated from it, as a form remembers what it submits", async () => {
    const { getByRole, getAllByTestId } = await renderPanel();
    const heightRow: HTMLElement = getAllByTestId("path-panel-row")[0] as HTMLElement;

    fireEvent.change(within(heightRow).getByRole("textbox", { name: "Height map" }), {
      target: { value: "C:\\work\\wall_height.png" },
    });

    await waitFor(() => expect(getByRole("button", { name: "Generate pair" })).toBeEnabled());

    // Typing alone records nothing: every keystroke would otherwise be remembered as a path.
    expect(within(heightRow).queryByRole("button", { name: "Recent paths" })).toBeNull();

    await userEvent.click(getByRole("button", { name: "Generate pair" }));

    await waitFor(() => expect(within(heightRow).getByRole("button", { name: "Recent paths" })).toBeInTheDocument());
  });

  it("warns before writing a pair DX9 cannot load, and asks the backend for that format", async () => {
    const { getByRole, getAllByTestId, getByTestId, queryByTestId } = await renderPanel();

    await waitFor(() => expect(getByRole("button", { name: "BC7" })).toBeInTheDocument());

    // The SDK's own format is chosen first and every renderer that was read loads it, so nothing is said.
    expect(queryByTestId("texture-bump-format-warning")).toBeNull();

    await userEvent.click(getByRole("button", { name: "BC7" }));

    expect(getByTestId("texture-bump-format-warning")).toHaveTextContent("not on DX9");

    fireEvent.change(
      within(getAllByTestId("path-panel-row")[0] as HTMLElement).getByRole("textbox", { name: "Height map" }),
      {
        target: { value: "C:\\work\\wall_height.png" },
      }
    );

    await waitFor(() => expect(getByRole("button", { name: "Generate pair" })).toBeEnabled());
    await userEvent.click(getByRole("button", { name: "Generate pair" }));

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith(
        ETexturesCommand.MAKE_BUMP,
        expect.objectContaining({ request: expect.objectContaining({ format: EGenerateBumpFormat.BC7 }) })
      )
    );
  });
});
