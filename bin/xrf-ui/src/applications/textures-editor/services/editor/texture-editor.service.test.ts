import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { EMPTY_TEXTURE_DESCRIPTOR_FORM } from "@/applications/textures-editor/lib/texture-descriptor-form";
import { TextureDescription, TextureDescriptorForm, TextureVocabulary } from "@/core/bindings/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";

import { TextureEditorService } from "./texture-editor.service";

const VOCABULARY: TextureVocabulary = {
  bumpModes: [{ label: "none", value: 0 }],
  flags: [{ bit: 1, label: "flGenerateMipMaps" }],
  formats: [{ label: "tfDXT1", value: 0 }],
  materials: [],
  mipFilters: [],
  textureTypes: [{ label: "Image", value: 0 }],
};

function form(overrides: Partial<TextureDescriptorForm> = {}): TextureDescriptorForm {
  return { ...EMPTY_TEXTURE_DESCRIPTOR_FORM, ...overrides };
}

/** A described texture that has a descriptor and a file to write it back to. */
function describedTexture(reference: string, overrides: Partial<TextureDescription> = {}): TextureDescription {
  return mockTextureDescription(reference, {
    form: form({ bumpName: "ston\\ston_beton05_bump" }),
    targets: {
      descriptor: { expected: { modifiedMs: 1, size: 2 }, path: `C:\\gamedata\\textures\\${reference}.thm` },
      texture: { expected: { modifiedMs: 1, size: 3 }, path: `C:\\gamedata\\textures\\${reference}.dds` },
    },
    ...overrides,
  });
}

function mockEditorService(): TextureEditorService {
  setMockInvokeResponses({
    ["plugin:textures|get_roots"]: null,
    ["plugin:textures|get_vocabulary"]: VOCABULARY,
  });

  const container: Container = mockContainer([JobsService, TextureSelectionService, TextureEditorService]);

  return container.get(TextureEditorService);
}

describe("TextureEditorService", () => {
  beforeEach(() => resetMockInvoke());

  it("applies its mobx annotations", () => {
    // A service whose annotations never got applied still passes every behavioural test here, because nothing in jest
    // reacts to its state - and then does nothing at all in the running app.
    const service: TextureEditorService = mockEditorService();

    expect(isObservableProp(service, "draft")).toBe(true);
    expect(isObservableProp(service, "vocabulary")).toBe(true);
    expect(isComputedProp(service, "isDirty")).toBe(true);
    expect(isComputedProp(service, "canSave")).toBe(true);
  });

  it("reads the vocabulary once when the editor opens", async () => {
    const service: TextureEditorService = mockEditorService();

    await service.onProvision();

    expect(service.vocabulary.value).toEqual(VOCABULARY);
  });

  it("starts clean when a texture is bound", () => {
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE));

    expect(service.draft?.bumpName).toBe("ston\\ston_beton05_bump");
    expect(service.isDirty).toBe(false);
  });

  it("becomes dirty on an edit and clean again on a discard", () => {
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE));
    service.edit({ bumpName: "ston\\other_bump" });

    expect(service.isDirty).toBe(true);
    expect(service.draft?.bumpName).toBe("ston\\other_bump");

    service.discard();

    expect(service.isDirty).toBe(false);
    expect(service.draft?.bumpName).toBe("ston\\ston_beton05_bump");
  });

  it("is clean again when a field is edited back to what it was", () => {
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE));
    service.edit({ detailScale: 0.5 });
    service.edit({ detailScale: EMPTY_TEXTURE_DESCRIPTOR_FORM.detailScale });

    expect(service.isDirty).toBe(false);
  });

  it("drops the draft when another texture is chosen", () => {
    // Switching nodes is what discards a draft; the editor prompts before letting that happen, and this is what the
    // prompt is protecting.
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE));
    service.edit({ bumpName: "ston\\other_bump" });
    service.bind(describedTexture("ston\\ston_beton06"));

    expect(service.isDirty).toBe(false);
    expect(service.draft?.bumpName).toBe("ston\\ston_beton05_bump");
  });

  it("keeps the draft when the same texture is described again", () => {
    // A save re-describes what it wrote, and a person may already be typing the next change into the form. Dropping
    // the draft there would silently undo it.
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE));
    service.edit({ bumpName: "ston\\other_bump" });
    service.bind(describedTexture(MOCK_TEXTURE));

    expect(service.draft?.bumpName).toBe("ston\\other_bump");
    expect(service.isDirty).toBe(true);
  });

  it("offers a form for a texture that has no descriptor yet", () => {
    // Most textures have none, so authoring one is an ordinary act rather than an error state.
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE, { form: null }));

    expect(service.draft).not.toBeNull();
    expect(service.isDirty).toBe(false);
  });

  it("refuses to save a texture served out of an archive", () => {
    // There is no file to replace, so the fields are readable and the save is not offered.
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE, { targets: null }));
    service.edit({ bumpName: "ston\\other_bump" });

    expect(service.isDirty).toBe(true);
    expect(service.canSave).toBe(false);
  });

  it("does not offer a save with nothing changed", () => {
    const service: TextureEditorService = mockEditorService();

    service.bind(describedTexture(MOCK_TEXTURE));

    expect(service.canSave).toBe(false);
  });

  it("has nothing to bind when nothing is selected", () => {
    const service: TextureEditorService = mockEditorService();

    service.bind(null);

    expect(service.draft).toBeNull();
    expect(service.isDirty).toBe(false);
    expect(service.canSave).toBe(false);
  });
});
