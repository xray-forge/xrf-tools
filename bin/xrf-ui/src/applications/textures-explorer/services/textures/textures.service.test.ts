import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { autorun, IReactionDisposer, isComputedProp, isObservableProp } from "@wirestate/mobx";

import { ETextureBadge, ITextureNode } from "@/applications/textures-explorer/lib/texture-catalog";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureCatalog } from "@/core/bindings/types/xrf-app";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  MOCK_BUMP,
  MOCK_COMPANION,
  MOCK_TEXTURE,
  mockBumpedTextureSummary,
  mockTextureCatalog,
  mockTextureDescription,
  mockTextureEntry,
  mockTextureRoots,
} from "@/fixtures/mocks/texture.mocks";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";

/** A root set holding one texture and the pair its descriptor declares. */
function mockBrowsedRoot(): TextureCatalog {
  return mockTextureCatalog([
    mockTextureEntry(MOCK_TEXTURE),
    mockTextureEntry(MOCK_BUMP),
    mockTextureEntry(MOCK_COMPANION),
  ]);
}

function mockOpenedService(overrides: Record<string, unknown> = {}): TexturesService {
  setMockInvokeResponses({
    ["plugin:textures|close"]: null,
    ["plugin:textures|describe_catalog"]: [mockBumpedTextureSummary()],
    ["plugin:textures|get_roots"]: null,
    ["plugin:textures|open"]: mockBrowsedRoot(),
    ...overrides,
  });

  const container: Container = mockContainer([TexturesService]);

  return container.get(TexturesService);
}

describe("TexturesService", () => {
  beforeEach(() => resetMockInvoke());

  it("applies its mobx annotations", () => {
    // A service whose annotations never got applied still passes every behavioural test here, because nothing in jest
    // reacts to its state - and then does nothing at all in the running app.
    const { service } = mockInjectedService(TexturesService);

    expect(isObservableProp(service, "catalog")).toBe(true);
    expect(isObservableProp(service, "summaries")).toBe(true);
    expect(isObservableProp(service, "selected")).toBe(true);
    expect(isObservableProp(service, "preview")).toBe(true);
    expect(isObservableProp(service, "isReady")).toBe(true);
    expect(isComputedProp(service, "isBrowsing")).toBe(true);
    expect(isComputedProp(service, "nodes")).toBe(true);
    expect(isComputedProp(service, "roots")).toBe(true);
    expect(isComputedProp(service, "selectedReference")).toBe(true);
  });

  it("is ready once the backend has been asked, answering or not", async () => {
    // The flag gates the whole screen behind a loader, so a provision that leaves it unset shows a spinner forever -
    // which is what a passing annotation test cannot notice.
    const restored: TexturesService = mockOpenedService({ ["plugin:textures|get_roots"]: mockTextureRoots() });

    expect(restored.isReady).toBe(false);

    await restored.onProvision();

    expect(restored.isReady).toBe(true);
    expect(restored.isBrowsing).toBe(true);

    const refused: TexturesService = mockOpenedService({
      ["plugin:textures|get_roots"]: () => {
        throw new Error("no backend here");
      },
    });

    await refused.onProvision();

    expect(refused.isReady).toBe(true);
    expect(refused.isBrowsing).toBe(false);
  });

  it("never opens the picker over a session that is coming back", async () => {
    // React's strict mode provisions twice, and the second restore cancels the first. A first attempt that announced
    // readiness on its way out would put the picker on screen for a quarter of a second over a session already being
    // restored, which is what a person sees as a flash of the wrong screen.
    const service: TexturesService = mockOpenedService({ ["plugin:textures|get_roots"]: mockTextureRoots() });
    const seen: Array<{ isBrowsing: boolean; isReady: boolean }> = [];
    const dispose: IReactionDisposer = autorun(() =>
      seen.push({ isBrowsing: service.isBrowsing, isReady: service.isReady })
    );

    await Promise.all([service.onProvision(), service.onProvision()]);

    dispose();

    expect(service.isReady).toBe(true);
    expect(service.isBrowsing).toBe(true);
    expect(seen.filter((it) => it.isReady && !it.isBrowsing)).toEqual([]);
  });

  it("lists a root and then reads every descriptor in it", async () => {
    const service: TexturesService = mockOpenedService();

    await service.openRoot("C:\\gamedata");

    expect(service.isBrowsing).toBe(true);
    expect(service.catalog.value?.entries).toHaveLength(3);
    expect(service.summaries.value).toHaveLength(1);

    // The listing has three entries and the sweep folds two of them away, which is what the tree draws.
    const nodes: Array<ITextureNode> = service.nodes;

    expect(nodes.map((node: ITextureNode) => node.reference)).toEqual([MOCK_TEXTURE]);
    expect(nodes[0].badges.has(ETextureBadge.BUMPED)).toBe(true);
  });

  it("leaves the tree browsable when the descriptor sweep fails", async () => {
    const service: TexturesService = mockOpenedService({
      ["plugin:textures|describe_catalog"]: () => {
        throw new Error("sweep failed");
      },
    });

    await service.openRoot("C:\\gamedata");

    expect(service.catalog.value?.entries).toHaveLength(3);
    expect(service.summaries.error?.message).toContain("sweep failed");
    // Unfolded, because nothing has said which of them belong together, but every row is still there.
    expect(service.nodes).toHaveLength(3);
  });

  it("reports a failed listing rather than an empty root", async () => {
    const service: TexturesService = mockOpenedService({
      ["plugin:textures|open"]: () => {
        throw new Error("root is unreadable");
      },
    });

    await service.openRoot("C:\\gamedata");

    expect(service.isBrowsing).toBe(false);
    expect(service.catalog.error?.message).toContain("root is unreadable");
  });

  it("describes the texture that was chosen, through the roots the listing used", async () => {
    const service: TexturesService = mockOpenedService({
      ["plugin:textures|describe"]: mockTextureDescription(),
    });

    await service.openRoot("C:\\gamedata");
    await service.select(MOCK_TEXTURE);

    expect(service.selectedReference).toBe(MOCK_TEXTURE);
    expect(service.selected.value?.material.outcome).toBe("flat");
  });

  it("keeps the descriptor on screen when the picture cannot be decoded", async () => {
    const service: TexturesService = mockOpenedService({
      ["plugin:textures|describe"]: mockTextureDescription(),
      ["plugin:textures|read_texture"]: () => {
        throw new Error("unsupported layout");
      },
    });

    await service.openRoot("C:\\gamedata");
    await service.select(MOCK_TEXTURE);

    expect(service.selected.value?.reference).toBe(MOCK_TEXTURE);
    expect(service.preview.error?.message).toContain("unsupported layout");
  });

  it("ends the whole session on close", async () => {
    const service: TexturesService = mockOpenedService({
      ["plugin:textures|describe"]: mockTextureDescription(),
    });

    await service.openRoot("C:\\gamedata");
    await service.select(MOCK_TEXTURE);
    await service.close();

    expect(service.isBrowsing).toBe(false);
    expect(service.selected.value).toBeNull();
    expect(service.nodes).toHaveLength(0);
  });

  it("refuses to describe a texture with nothing open", async () => {
    const service: TexturesService = mockOpenedService({
      ["plugin:textures|describe"]: mockTextureDescription(),
    });

    await service.select(MOCK_TEXTURE);

    expect(service.selected.value).toBeNull();
  });
});
