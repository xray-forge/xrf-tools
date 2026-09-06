import { describe, expect, it } from "@jest/globals";
import { act, fireEvent, RenderResult, waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { TextureEncodingService } from "@/applications/textures-editor/services/encoding";
import { TextureDescription, TextureVocabulary } from "@/core/bindings/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { TextureDescriptorPanel } from "./TextureDescriptorPanel";

const VOCABULARY: TextureVocabulary = {
  bumpModes: [
    { label: "none", value: 0 },
    { label: "use", value: 1 },
  ],
  flags: [
    { bit: 1, label: "flGenerateMipMaps" },
    { bit: 1 << 25, label: "flHasAlpha" },
  ],
  formats: [
    { label: "tfDXT1", value: 0 },
    { label: "tfDXT5", value: 4 },
  ],
  materials: [{ label: "mtOrenNayar_Blin", value: 0 }],
  mipFilters: [{ label: "kMIPFilterBox", value: 1 }],
  textureTypes: [
    { label: "ttImage", value: 0 },
    { label: "ttTerrain", value: 4 },
  ],
};

async function renderPanel(
  description: TextureDescription
): Promise<{ render: RenderResult; service: TextureEditorService }> {
  resetMockInvoke();

  setMockInvokeResponses({
    ["plugin:textures|describe"]: description,
    ["plugin:textures|get_roots"]: null,
    ["plugin:textures|get_vocabulary"]: VOCABULARY,
  });

  const container: Container = mockContainer([
    JobsService,
    TextureSelectionService,
    TextureEncodingService,
    TextureEditorService,
  ]);
  const service: TextureEditorService = container.get(TextureEditorService);

  await container.get(TextureSelectionService).openFile("C:\\gamedata\\textures\\ston\\ston_beton05.dds");

  const render: RenderResult = renderWithProviders(<TextureDescriptorPanel />, { container });

  // The render provisions the container, and the vocabulary it asks for arrives a microtask later. Settling that
  // inside `act` is what keeps the panel's first real paint a React update the test owns.
  await settle();

  return { render, service };
}

/** Let every already-resolved promise land, with React holding the batch. */
async function settle(): Promise<void> {
  await act(async () => {
    await Promise.resolve();
  });
}

function describedTexture(overrides: Partial<TextureDescription> = {}): TextureDescription {
  return mockTextureDescription(MOCK_TEXTURE, {
    form: {
      borderColor: 0,
      bumpMode: 1,
      bumpName: "ston\\ston_beton05_bump",
      detailName: "",
      detailScale: 1,
      extNormalMapName: "",
      fadeAmount: 0,
      fadeColor: 0,
      fadeDelay: 0,
      flags: 1,
      format: 4,
      height: 512,
      material: 0,
      materialWeight: 0,
      mipFilter: 1,
      textureType: 0,
      virtualHeight: 0.05,
      width: 512,
    },
    targets: {
      descriptor: { expected: { modifiedMs: 1, size: 2 }, path: "C:\\gamedata\\textures\\ston\\ston_beton05.thm" },
      texture: { expected: { modifiedMs: 1, size: 3 }, path: "C:\\gamedata\\textures\\ston\\ston_beton05.dds" },
    },
    ...overrides,
  });
}

describe("TextureDescriptorPanel", () => {
  it("shows the descriptor's own values under the names the sdk gives them", async () => {
    const { render } = await renderPanel(describedTexture());

    await waitFor(() => expect(render.getByTestId("texture-field-format")).toBeTruthy());

    // The stored numbers are shown as `tfDXT5` and `ttImage` rather than as 4 and 0, and the flag word as its bits.
    expect(render.getByTestId("texture-field-format").textContent).toContain("tfDXT5");
    expect(render.getByTestId("texture-field-texture-type").textContent).toContain("ttImage");
    expect(
      (render.getByTestId("texture-flag-flGenerateMipMaps").querySelector("input") as HTMLInputElement).checked
    ).toBe(true);
    expect((render.getByTestId("texture-flag-flHasAlpha").querySelector("input") as HTMLInputElement).checked).toBe(
      false
    );
  });

  it("keeps save and discard closed until something is changed", async () => {
    const { render, service } = await renderPanel(describedTexture());

    await waitFor(() => expect(render.getByTestId("texture-descriptor-save")).toBeTruthy());

    expect((render.getByTestId("texture-descriptor-save") as HTMLButtonElement).disabled).toBe(true);

    service.edit({ bumpName: "ston\\other_bump" });

    await waitFor(() =>
      expect((render.getByTestId("texture-descriptor-save") as HTMLButtonElement).disabled).toBe(false)
    );
    expect((render.getByTestId("texture-descriptor-discard") as HTMLButtonElement).disabled).toBe(false);
  });

  it("writes a typed name into the draft", async () => {
    const { render, service } = await renderPanel(describedTexture());

    await waitFor(() => expect(render.getByTestId("texture-field-bump-name")).toBeTruthy());

    fireEvent.change(render.getByTestId("texture-field-bump-name").querySelector("input") as HTMLInputElement, {
      target: { value: "ston\\other_bump" },
    });

    await waitFor(() => expect(service.draft?.bumpName).toBe("ston\\other_bump"));
    expect(service.isDirty).toBe(true);
  });

  it("sets one flag bit without touching the rest of the word", async () => {
    const { render, service } = await renderPanel(describedTexture());

    await waitFor(() => expect(render.getByTestId("texture-flag-flHasAlpha")).toBeTruthy());

    await act(async () =>
      fireEvent.click(render.getByTestId("texture-flag-flHasAlpha").querySelector("input") as HTMLInputElement)
    );

    // `flGenerateMipMaps` was set and stays set.
    await waitFor(() => expect(service.draft?.flags).toBe(1 | (1 << 25)));
  });

  it("shows width and height read-only, because the dds header is the authority", async () => {
    const { render } = await renderPanel(describedTexture());

    await waitFor(() => expect(render.getByTestId("texture-field-width")).toBeTruthy());

    expect((render.getByTestId("texture-field-width").querySelector("input") as HTMLInputElement).readOnly).toBe(true);
    expect((render.getByTestId("texture-field-height").querySelector("input") as HTMLInputElement).readOnly).toBe(true);
  });

  it("says a texture out of an archive cannot be written, and offers no save", async () => {
    const { render, service } = await renderPanel(describedTexture({ targets: null }));

    await waitFor(() => expect(render.getByTestId("texture-descriptor-save")).toBeTruthy());

    service.edit({ bumpName: "ston\\other_bump" });

    await waitFor(() =>
      expect((render.getByTestId("texture-descriptor-save") as HTMLButtonElement).disabled).toBe(true)
    );
    expect(render.container.textContent).toContain("served out of an archive");
  });

  it("offers to author a descriptor for a texture that has none", async () => {
    const { render } = await renderPanel(describedTexture({ form: null }));

    await waitFor(() => expect(render.getByTestId("texture-field-format")).toBeTruthy());

    expect(render.container.textContent).toContain("No .thm sits beside this texture");
  });

  it("says what it would show when nothing is selected", async () => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:textures|get_roots"]: null, ["plugin:textures|get_vocabulary"]: VOCABULARY });

    const container: Container = mockContainer([
      JobsService,
      TextureSelectionService,
      TextureEncodingService,
      TextureEditorService,
    ]);
    const render: RenderResult = renderWithProviders(<TextureDescriptorPanel />, { container });

    await settle();

    expect(render.container.textContent).toContain("No texture selected");
  });
});
