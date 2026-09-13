import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";

import { TextureEditorService } from "@/applications/textures-editor/services/editor";
import { TextureEncodingService } from "@/applications/textures-editor/services/encoding";
import { TextureMakeBumpOutcome } from "@/core/ipc/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription, mockTextureVocabulary } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";

import { ITextureBumpSources, TextureBumpService } from "./texture-bump.service";

const SOURCES: ITextureBumpSources = {
  gloss: null,
  glossConstant: 0.5,
  height: "C:\\work\\wall_height.png",
  normalMap: null,
  virtualHeight: 0.05,
};

function mockOutcome(overrides: Partial<TextureMakeBumpOutcome> = {}): TextureMakeBumpOutcome {
  return {
    bump: "C:\\gamedata\\textures\\ston\\ston_beton05_bump.dds",
    companion: "C:\\gamedata\\textures\\ston\\ston_beton05_bump#.dds",
    glossPower: 0.5,
    isGlossTooDark: false,
    outcome: "completed",
    ...overrides,
  };
}

function describedTexture(overrides = {}) {
  return mockTextureDescription(MOCK_TEXTURE, {
    targets: {
      descriptor: { expected: null, path: "C:\\gamedata\\textures\\ston\\ston_beton05.thm" },
      texture: { expected: { modifiedMs: 1, size: 3 }, path: "C:\\gamedata\\textures\\ston\\ston_beton05.dds" },
    },
    ...overrides,
  });
}

async function mockService(outcome: TextureMakeBumpOutcome = mockOutcome()): Promise<{
  bumpService: TextureBumpService;
  editorService: TextureEditorService;
}> {
  setMockInvokeResponses({
    ["plugin:textures|describe"]: describedTexture(),
    ["plugin:textures|get_vocabulary"]: mockTextureVocabulary(),
    ["plugin:textures|make_bump"]: outcome,
    ["plugin:textures|read_texture"]: new ArrayBuffer(0),
  });

  const container: Container = mockContainer([
    JobsService,
    TextureSelectionService,
    TextureEncodingService,
    TextureEditorService,
    TextureBumpService,
  ]);
  const editorService: TextureEditorService = container.get(TextureEditorService);

  await editorService.onProvision();
  await container.get(TextureSelectionService).openFile("C:\\gamedata\\textures\\ston\\ston_beton05.dds");

  editorService.bind(describedTexture());

  return { bumpService: container.get(TextureBumpService), editorService };
}

describe("TextureBumpService", () => {
  beforeEach(() => resetMockInvoke());

  it("writes the pair beside the texture, without being told where", async () => {
    // The pair belongs to the texture that is open and its two names are derived, so there is no destination to get
    // wrong. The backend appends `_bump` and `_bump#` to what this answers.
    const { bumpService } = await mockService();

    expect(bumpService.destination).toBe("C:\\gamedata\\textures\\ston\\ston_beton05");
    expect(bumpService.canGenerate).toBe(true);
  });

  it("cannot generate for a texture served out of an archive", async () => {
    setMockInvokeResponses({
      ["plugin:textures|describe"]: describedTexture({ targets: null }),
      ["plugin:textures|get_vocabulary"]: mockTextureVocabulary(),
      ["plugin:textures|read_texture"]: new ArrayBuffer(0),
    });

    const container: Container = mockContainer([
      JobsService,
      TextureSelectionService,
      TextureEncodingService,
      TextureEditorService,
      TextureBumpService,
    ]);

    await container.get(TextureSelectionService).openFile("C:\\gamedata\\textures\\ston\\ston_beton05.dds");

    expect(container.get(TextureBumpService).destination).toBeNull();
    expect(container.get(TextureBumpService).canGenerate).toBe(false);
  });

  it("points the descriptor at the pair it wrote, and leaves the node dirty", async () => {
    // The pair is on disk the moment the run finishes; the descriptor naming it is not. That gap is the one state a
    // generation leaves behind, and it is an ordinary save away from closing.
    const { bumpService, editorService } = await mockService();

    expect(editorService.isDirty).toBe(false);

    await bumpService.run(SOURCES);

    expect(editorService.draft?.bumpName).toBe(`${MOCK_TEXTURE}_bump`);
    expect(editorService.draft?.bumpMode).toBe(mockTextureVocabulary().bumpModeUse);
    expect(editorService.isDirty).toBe(true);
  });

  it("touches the descriptor for a run that was stopped", async () => {
    // A cancelled generation wrote neither half, so pointing a descriptor at a pair that is not there would be worse
    // than doing nothing.
    const { bumpService, editorService } = await mockService(mockOutcome({ outcome: "cancelled" }));

    await bumpService.run(SOURCES);

    expect(editorService.draft?.bumpName).toBe("");
    expect(editorService.isDirty).toBe(false);
  });
});
