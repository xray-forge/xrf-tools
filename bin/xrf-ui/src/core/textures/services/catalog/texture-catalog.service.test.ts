import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";

import { TextureCatalogMode } from "@/core/bindings/types/xrf-app";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  MOCK_TEXTURE,
  mockTextureCatalog,
  mockTextureDescription,
  mockTextureEntry,
  mockTextureSummary,
} from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";

import { TextureCatalogService } from "./texture-catalog.service";

/** What each command was asked for, since the mode is the whole of what these cases are about. */
interface IAsked {
  openModes: Array<TextureCatalogMode>;
  sweeps: number;
  describes: Array<unknown>;
}

function mockService(): { service: TextureCatalogService; asked: IAsked } {
  const asked: IAsked = { describes: [], openModes: [], sweeps: 0 };

  setMockInvokeResponses({
    ["plugin:textures|describe"]: (args?: Record<string, unknown>) => {
      asked.describes.push(args?.source);

      return mockTextureDescription();
    },
    ["plugin:textures|describe_catalog"]: () => {
      asked.sweeps += 1;

      return [mockTextureSummary(MOCK_TEXTURE)];
    },
    ["plugin:textures|get_session"]: null,
    ["plugin:textures|open"]: (args?: Record<string, unknown>) => {
      const mode: TextureCatalogMode = args?.mode as TextureCatalogMode;

      asked.openModes.push(mode);

      return mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)], { mode });
    },
    ["plugin:textures|read_texture"]: new ArrayBuffer(0),
  });

  const container: Container = mockContainer([TextureSelectionService, TextureCatalogService]);

  return { asked, service: container.get(TextureCatalogService) };
}

describe("TextureCatalogService", () => {
  beforeEach(() => resetMockInvoke());

  it("lists a game tree by reference, and sweeps its descriptors", async () => {
    const { service, asked } = mockService();

    await service.openRoot("C:\\gamedata");

    expect(asked.openModes).toEqual(["roots"]);
    expect(asked.sweeps).toBe(1);
    expect(service.nodes).toHaveLength(1);
  });

  it("lists a loose folder without sweeping it", async () => {
    // The sweep reads descriptors by engine reference, and a loose listing has none. Running it anyway would hold the
    // tree behind a pass with nothing to say about it.
    const { service, asked } = mockService();

    await service.openLooseDirectory("C:\\work\\my-textures");

    expect(asked.openModes).toEqual(["looseDirectory"]);
    expect(asked.sweeps).toBe(0);
    expect(service.isBrowsing).toBe(true);
  });

  it("opens a row by the address the listing put on it", async () => {
    const { service, asked } = mockService();

    await service.openLooseDirectory("C:\\work\\my-textures");
    await service.select({ kind: "file", path: "C:\\work\\my-textures\\brick01.dds" });

    expect(asked.describes).toEqual([{ kind: "file", path: "C:\\work\\my-textures\\brick01.dds" }]);
  });
});
