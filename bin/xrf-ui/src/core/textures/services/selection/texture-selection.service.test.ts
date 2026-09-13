import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";

import { createRoots } from "@/core/assets/lib";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

const TEXTURE: string = "C:\\mods\\mine\\gamedata\\textures\\wall.dds";
const INSTALLATION: string = "C:\\Games\\stalker";

describe("TextureSelectionService", () => {
  /** The roots each describe was addressed by, which is the whole of what these cases are about. */
  function mockService(): { asked: Array<Nullable<XrayRoots>>; service: TextureSelectionService } {
    const asked: Array<Nullable<XrayRoots>> = [];

    setMockInvokeResponses({
      ["plugin:textures|describe"]: (parameters?: Record<string, unknown>) => {
        asked.push((parameters as { roots: Nullable<XrayRoots> })?.roots ?? null);

        return mockTextureDescription();
      },
    });

    const container: Container = mockContainer([TextureSelectionService]);

    return { asked, service: container.get(TextureSelectionService) };
  }

  beforeEach(() => resetMockInvoke());

  it("resolves a loose file in its own neighbourhood when no further tree was named", async () => {
    const { asked, service } = mockService();

    await service.openFile(TEXTURE);

    // Centred on the file, and nothing behind it: with no ambient set anywhere, reading only what you opened is the
    // honest default rather than a silent failure to layer.
    expect(asked).toEqual([createRoots([], TEXTURE)]);
  });

  it("resolves a loose file against the tree the surface was opened with", async () => {
    const { asked, service } = mockService();

    service.setAssetRoot(INSTALLATION);

    await service.openFile(TEXTURE);

    // What replaced the configured game data: a mod tree's texture still finds the bump pair the base game holds,
    // because the open said where to look rather than a setting somewhere else.
    expect(asked).toEqual([createRoots([INSTALLATION], TEXTURE)]);
  });

  it("leaves a listing's own roots alone, because a browsed row is already addressed", async () => {
    const { asked, service } = mockService();
    const browsed: XrayRoots = createRoots(["C:\\mods\\mine\\gamedata", INSTALLATION]);

    service.setAssetRoot(INSTALLATION);

    await service.openReference("textures\\wall", browsed);

    expect(asked).toEqual([browsed]);
  });
});
