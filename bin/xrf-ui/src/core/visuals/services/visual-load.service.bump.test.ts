import { beforeEach, describe, expect, it } from "@jest/globals";

import { createRoots } from "@/core/assets/lib";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { XrayAsset, XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockDdsFile, mockDx10DdsFile } from "@/fixtures/mocks/dds.mocks";
import { InvokeHandler, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockMaterialDescriptor,
  mockPackedSubmesh,
  mockSelectedVisual,
  mockTextureDependency,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { muteConsole } from "@/fixtures/utils/console";
import { mockInjectedService } from "@/fixtures/utils/container";

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);
const ENTRY: string = "meshes\\actors\\stalker.ogf";
const REFERENCE: string = "wpn\\wpn_ak74";
const BASE: string = "textures\\wpn\\wpn_ak74.dds";
const BUMP: string = "textures\\wpn\\wpn_ak74_bump.dds";
const COMPANION: string = "textures\\wpn\\wpn_ak74_bump#.dds";
const DUMMY_BUMP: string = "textures\\ed\\ed_dummy_bump.dds";
const DUMMY_COMPANION: string = "textures\\ed\\ed_dummy_bump#.dds";

function mockAsset(logicalPath: string): XrayAsset {
  return { container: { kind: "directory", relativePath: logicalPath, root: "C:\\gamedata" }, logicalPath };
}

/**
 * A visual with two submeshes declaring one texture, whose material is `material`.
 *
 * @param material - What the backend resolved for the shared reference.
 * @returns The description and the geometry buffer it addresses.
 */
function mockBumpedVisual(material: XrayMaterialDescriptor): {
  selected: SelectedVisualDescription;
  buffer: ArrayBuffer;
} {
  const buffer: MockVisualBuffer = new MockVisualBuffer();

  return {
    selected: mockSelectedVisual({
      source: { kind: "asset", logicalPath: ENTRY },
      roots: ROOTS,
      description: mockVisualDescription({
        submeshes: [mockPackedSubmesh(buffer), mockPackedSubmesh(buffer, { index: 1 })],
        bufferLength: buffer.byteLength,
      }),
      dependencies: {
        motions: [],
        textures: [mockTextureDependency({ submeshIndex: 0 }), mockTextureDependency({ submeshIndex: 1 })],
      },
      materials: { [REFERENCE]: material },
    }),
    buffer: buffer.toArrayBuffer(),
  };
}

/** The dummy outcome: both halves substituted by the engine's flat pair. */
function mockDummyMaterial(): XrayMaterialDescriptor {
  const material: XrayMaterialDescriptor = mockMaterialDescriptor({ outcome: "dummy" });

  material.bump!.bump.resolution = {
    kind: "substituted",
    step: "asset root",
    fallback: "ed\\ed_dummy_bump",
    assets: [mockAsset(DUMMY_BUMP)],
  };
  material.bump!.companion.resolution = {
    kind: "substituted",
    step: "asset root",
    fallback: "ed\\ed_dummy_bump#",
    assets: [mockAsset(DUMMY_COMPANION)],
  };

  return material;
}

describe("VisualLoadService bump pairs", () => {
  // One test hands the renderer's loader a layout it has no branch for, and it reports the refusal itself.
  muteConsole("error");

  beforeEach(() => {
    resetMockInvoke();
  });

  it("reads and uploads a pair once for every submesh sharing it, beside the base texture", async () => {
    const { selected, buffer } = mockBumpedVisual(mockMaterialDescriptor());
    const reads: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: ((args) => {
        reads.push(String(args?.logicalPath));

        return mockDdsFile({ fourCC: "DXT5" });
      }) as InvokeHandler,
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(reads.sort()).toEqual([BASE, BUMP, COMPANION].sort());
    expect(service.bumps.size).toBe(2);
    expect(service.bumps.get(0)?.bump).toBe(service.bumps.get(1)?.bump);
    expect(service.bumpStatuses.get(0)).toEqual({
      submeshIndex: 0,
      bump: EVisualTextureState.APPLIED,
      companion: EVisualTextureState.APPLIED,
      reason: null,
    });
  });

  it("uploads the engine's real dummy pair for a dummy outcome, so the preview shows what the game shows", async () => {
    const { selected, buffer } = mockBumpedVisual(mockDummyMaterial());
    const reads: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: ((args) => {
        reads.push(String(args?.logicalPath));

        return mockDdsFile({ fourCC: "DXT5" });
      }) as InvokeHandler,
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(reads.sort()).toEqual([DUMMY_BUMP, DUMMY_COMPANION, BASE].sort());
    expect(service.bumps.get(0)).toBeDefined();
  });

  it("keeps the base texture and names the half that failed when one input cannot be uploaded", async () => {
    // The pair is all or nothing on the surface, since the sampler reads both; the report is per half, since the fix
    // is.
    const { selected, buffer } = mockBumpedVisual(mockMaterialDescriptor());

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: ((args) => {
        // BC7 is a layout the renderer's loader has no branch for, and it logs the refusal itself.
        return args?.logicalPath === COMPANION ? mockDx10DdsFile(98) : mockDdsFile({ fourCC: "DXT5" });
      }) as InvokeHandler,
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.APPLIED);
    expect(service.bumps.size).toBe(0);
    expect(service.bumpStatuses.get(0)).toEqual({
      submeshIndex: 0,
      bump: EVisualTextureState.APPLIED,
      companion: EVisualTextureState.UNSUPPORTED_FORMAT,
      reason: null,
    });
  });

  it("frees the pair with the base textures when the visual is cleared", async () => {
    const { selected, buffer } = mockBumpedVisual(mockMaterialDescriptor());

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockDdsFile({ fourCC: "DXT5" }),
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    const pair = service.bumps.get(0);
    let disposed: number = 0;

    pair?.bump.addEventListener("dispose", () => (disposed += 1));
    pair?.companion.addEventListener("dispose", () => (disposed += 1));

    service.clear();

    expect(disposed).toBe(2);
    expect(service.bumps.size).toBe(0);
    expect(service.bumpStatuses.size).toBe(0);
  });
});
