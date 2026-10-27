import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { EAssetsBulkRoute } from "@xrf/ipc/commands/assets-bulk";
import { ETexturesCommand } from "@xrf/ipc/commands/textures";
import { mockIpcBulkRoutes, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { TextureCatalogMode } from "@xrf/ipc/types/xrf-app-textures";

import { createRoots } from "@/core/assets/lib";
import {
  MOCK_TEXTURE,
  mockTextureCatalog,
  mockTextureDescription,
  mockTextureEntry,
  mockTextureSummary,
} from "@/core/textures/fixtures/texture.mocks";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
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

  mockIpcCommands({
    [ETexturesCommand.DESCRIBE]: (args?: Record<string, unknown>) => {
      asked.describes.push(args?.source);

      return mockTextureDescription();
    },
    [ETexturesCommand.DESCRIBE_CATALOG]: () => {
      asked.sweeps += 1;

      return [mockTextureSummary(MOCK_TEXTURE)];
    },
    [ETexturesCommand.RESTORE_BROWSE]: mockSessionResponse(null),
    [ETexturesCommand.OPEN_BROWSE]: mockSessionResponse((args?: Record<string, unknown>) => {
      const mode: TextureCatalogMode = args?.mode as TextureCatalogMode;

      asked.openModes.push(mode);

      return mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)], { mode });
    }),
  });

  mockIpcBulkRoutes({
    [EAssetsBulkRoute.READ_TEXTURE]: new ArrayBuffer(0),
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

  it("lists what the tool still browsed again as it is restored, letting the restored opening go", async () => {
    const { service, asked } = mockService();
    const closed: Array<unknown> = [];

    mockIpcCommands({
      [ETexturesCommand.DESCRIBE_CATALOG]: [mockTextureSummary(MOCK_TEXTURE)],
      [ETexturesCommand.RESTORE_BROWSE]: mockSessionSnapshot(
        { mode: "looseDirectory", roots: createRoots(["C:\\work\\my-textures"]) },
        "restored"
      ),
      [ETexturesCommand.OPEN_BROWSE]: mockSessionResponse((args?: Record<string, unknown>) => {
        asked.openModes.push(args?.mode as TextureCatalogMode);

        return mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)], { mode: args?.mode as TextureCatalogMode });
      }),
      [ETexturesCommand.CLOSE_BROWSE]: (args?: Record<string, unknown>) => void closed.push(args?.sessionIds),
    });

    await service.onProvision();

    expect(asked.openModes).toEqual(["looseDirectory"]);
    expect(service.catalog.isRestored).toBe(true);
    expect(service.isBrowsing).toBe(true);
    expect(service.catalog.sessionId).not.toBe("restored");
    expect(closed).toEqual([["restored"]]);
  });

  it("opens a row by the address the listing put on it", async () => {
    const { service, asked } = mockService();

    await service.openLooseDirectory("C:\\work\\my-textures");
    await service.select({ kind: "file", path: "C:\\work\\my-textures\\brick01.dds" });

    expect(asked.describes).toEqual([{ kind: "file", path: "C:\\work\\my-textures\\brick01.dds" }]);
  });
});
